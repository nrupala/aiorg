use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Scope {
    root: PathBuf,
    allowed: Vec<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct PatchPlan {
    pub summary: String,
    pub edits: Vec<PatchEdit>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub struct PatchEdit {
    pub path: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct AppliedPatch {
    target: PathBuf,
    backup: Option<PathBuf>,
}

impl Scope {
    pub fn new(root: &Path, allowed: &[PathBuf]) -> Result<Self> {
        let root = root.canonicalize()?;
        let mut paths = Vec::new();
        for path in allowed {
            paths.push(
                root.join(path)
                    .canonicalize()
                    .unwrap_or_else(|_| root.join(path)),
            );
        }
        Ok(Self {
            root,
            allowed: paths,
        })
    }

    pub fn resolve(&self, relative: &Path) -> Result<PathBuf> {
        if relative.is_absolute() {
            bail!("absolute paths are forbidden")
        }
        if relative
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            bail!("parent traversal is forbidden")
        }
        let first = relative
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if first == ".git" || first == ".aiorg" {
            bail!("protected path is not editable: {}", relative.display())
        }
        let candidate = self.root.join(relative);
        let canonical = candidate.canonicalize().unwrap_or(candidate.clone());
        if canonical == self.root || self.allowed.iter().any(|p| canonical.starts_with(p)) {
            Ok(canonical)
        } else {
            bail!("path is outside the role scope: {}", relative.display())
        }
    }

    pub fn apply(&self, plan: &PatchPlan, backup_root: &Path) -> Result<Vec<AppliedPatch>> {
        if plan.edits.is_empty() || plan.edits.len() > 32 {
            bail!("patch must contain 1..32 edits")
        }
        std::fs::create_dir_all(backup_root)?;
        let mut applied = Vec::new();
        for (index, edit) in plan.edits.iter().enumerate() {
            if edit.content.len() > 2_000_000 {
                bail!("edit is too large: {}", edit.path)
            }
            let target = self.resolve(Path::new(&edit.path))?;
            let backup = if target.exists() {
                let path = backup_root.join(format!("{index}.bak"));
                std::fs::copy(&target, &path)
                    .with_context(|| format!("backup {}", target.display()))?;
                Some(path)
            } else {
                None
            };
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if let Err(error) = std::fs::write(&target, edit.content.as_bytes()) {
                let partial = AppliedPatch {
                    target: target.clone(),
                    backup: backup.clone(),
                };
                let _ = rollback(&[partial]);
                return Err(error.into());
            }
            applied.push(AppliedPatch { target, backup });
        }
        Ok(applied)
    }
}

pub fn rollback(applied: &[AppliedPatch]) -> Result<()> {
    for item in applied.iter().rev() {
        if let Some(backup) = &item.backup {
            std::fs::copy(backup, &item.target)?;
        } else if item.target.exists() {
            std::fs::remove_file(&item.target)?;
        }
    }
    Ok(())
}

pub fn commit_backup(applied: &[AppliedPatch]) -> Result<()> {
    for item in applied {
        if let Some(backup) = &item.backup {
            if backup.exists() {
                std::fs::remove_file(backup)?;
            }
        }
    }
    Ok(())
}

pub fn parse_patch(raw: &str) -> Result<PatchPlan> {
    let plan: PatchPlan =
        serde_json::from_str(raw).context("engineer response was not strict patch JSON")?;
    if plan.summary.trim().is_empty() {
        bail!("patch summary is empty")
    }
    Ok(plan)
}

pub fn run_bwrap(project: &Path, command: &str) -> Result<i32> {
    let project = project.canonicalize()?;
    let wsl_path = wsl_path(&project)?;
    let script = format!("set -eu; cd /tmp/workspace; {command}");
    let status = Command::new("wsl.exe")
        .args([
            "-e",
            "bwrap",
            "--die-with-parent",
            "--unshare-pid",
            "--unshare-net",
            "--dir",
            "/tmp",
            "--ro-bind",
            "/",
            "/",
            "--tmpfs",
            "/tmp",
            "--dir",
            "/tmp/workspace",
            "--bind",
            &wsl_path,
            "/tmp/workspace",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--",
            "bash",
            "-lc",
            &script,
        ])
        .status()?;
    Ok(status.code().unwrap_or(1))
}

fn wsl_path(path: &Path) -> Result<String> {
    let output = Command::new("wsl.exe")
        .args(["-e", "wslpath", "-a", &path.to_string_lossy()])
        .output()?;
    if !output.status.success() {
        bail!("wslpath failed")
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_rejects_escape_and_protected_paths() {
        let dir = tempfile::tempdir().unwrap();
        let scope = Scope::new(dir.path(), &[PathBuf::from(".")]).unwrap();
        assert!(scope.resolve(Path::new("src/main.rs")).is_ok());
        assert!(scope.resolve(Path::new("../secret")).is_err());
        assert!(scope.resolve(Path::new(".git/config")).is_err());
        assert!(scope.resolve(Path::new(".aiorg/run.json")).is_err());
    }
    #[test]
    fn patch_roundtrip_and_rollback() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("src");
        std::fs::create_dir(&target).unwrap();
        let file = target.join("main.rs");
        std::fs::write(&file, "old").unwrap();
        let scope = Scope::new(dir.path(), &[PathBuf::from(".")]).unwrap();
        let plan = PatchPlan {
            summary: "fix".into(),
            edits: vec![PatchEdit {
                path: "src/main.rs".into(),
                content: "new".into(),
            }],
        };
        let applied = scope.apply(&plan, &dir.path().join("backups")).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "new");
        rollback(&applied).unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "old");
    }
    #[test]
    fn malformed_patch_rejected() {
        assert!(parse_patch("not-json").is_err());
    }
}
