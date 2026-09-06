use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Scope {
    root: PathBuf,
    allowed: Vec<PathBuf>,
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
        let candidate = self.root.join(relative);
        let canonical = candidate.canonicalize().unwrap_or(candidate.clone());
        if canonical == self.root || self.allowed.iter().any(|p| canonical.starts_with(p)) {
            Ok(canonical)
        } else {
            bail!("path is outside the role scope: {}", relative.display())
        }
    }
    pub fn read(&self, relative: &Path) -> Result<Vec<u8>> {
        Ok(std::fs::read(self.resolve(relative)?)?)
    }
    pub fn write(&self, relative: &Path, bytes: &[u8]) -> Result<()> {
        let path = self.resolve(relative)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, bytes)?;
        Ok(())
    }
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
            "--ro-bind",
            "/",
            "/",
            "--tmpfs",
            "/tmp",
            "--dir",
            "/tmp/workspace",
            "--bind",
            "/",
            "/",
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
    fn scope_rejects_escape() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("src")).unwrap();
        let scope = Scope::new(dir.path(), &[PathBuf::from("src")]).unwrap();
        assert!(scope.resolve(Path::new("src/main.rs")).is_ok());
        assert!(scope.resolve(Path::new("../secret")).is_err());
    }
}
