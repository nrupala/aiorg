#!/usr/bin/env node
// AIORG bump-version (App Versioning Standard, non-JS variant).
// Usage: node scripts/bump-version.mjs X.Y.Z
// Writes VERSION, syncs workspace Cargo version, promotes CHANGELOG [Unreleased].
import { readFileSync, writeFileSync } from 'node:fs';

const ver = process.argv[2];
if (!ver || !/^\d+\.\d+\.\d+$/.test(ver)) {
  console.error('usage: node scripts/bump-version.mjs X.Y.Z');
  process.exit(1);
}
const [, minor, patch] = ver.split('.').map(Number);
if (minor >= 100 || patch >= 100) {
  console.error('MINOR and PATCH must stay < 100 (versionCode-safe rule)');
  process.exit(1);
}

const oldVer = readFileSync('VERSION', 'utf8').trim();
writeFileSync('VERSION', ver + '\n');

const cargoPath = 'engine/Cargo.toml';
const cargo = readFileSync(cargoPath, 'utf8');
if (!/version = "\d+\.\d+\.\d+"/.test(cargo)) {
  console.error(`FATAL: no workspace version line found in ${cargoPath}`);
  process.exit(1);
}
writeFileSync(
  cargoPath,
  cargo.replace(/(version = ")\d+\.\d+\.\d+(")/, `$1${ver}$2`)
);

const clPath = 'CHANGELOG.md';
let cl = readFileSync(clPath, 'utf8');
const today = new Date().toISOString().slice(0, 10);
if (!cl.includes('## [Unreleased]')) {
  console.error('FATAL: CHANGELOG.md has no [Unreleased] section');
  process.exit(1);
}
cl = cl.replace(
  '## [Unreleased]',
  `## [Unreleased]\n\n## [${ver}] - ${today}`
);
writeFileSync(clPath, cl);

console.log(`bumped ${oldVer} -> ${ver}`);
console.log('  VERSION written; engine/Cargo.toml synced; CHANGELOG promoted.');
console.log(`next: git add -A && git commit -m "chore(release): ${ver}" && git tag v${ver} && git push --follow-tags`);
