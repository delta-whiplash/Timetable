#!/usr/bin/env node

import { execSync } from 'child_process';
import { readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

const args = process.argv.slice(2);
const releaseType = args[0] || 'patch';

if (!['patch', 'minor', 'major'].includes(releaseType)) {
  console.error('Usage: node release.js [patch|minor|major]');
  process.exit(1);
}

// Helper function to execute commands
function exec(command) {
  console.log(`> ${command}`);
  return execSync(command, { stdio: 'inherit' });
}

// Check if on main branch
const currentBranch = execSync('git rev-parse --abbrev-ref HEAD').toString().trim();
if (currentBranch !== 'main') {
  console.error('Error: Must be on main branch');
  process.exit(1);
}

// Check for uncommitted changes
const status = execSync('git status --porcelain').toString().trim();
if (status) {
  console.error('Error: Working directory not clean');
  console.error('Please commit or stash your changes first');
  process.exit(1);
}

// Read current version from package.json
const packageJsonPath = join(process.cwd(), 'package.json');
const packageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
const currentVersion = packageJson.version;

console.log(`\nCurrent version: ${currentVersion}`);
console.log(`Release type: ${releaseType}\n`);

// Bump version using npm
exec(`npm version ${releaseType} --no-git-tag-version`);

// Read new version
const updatedPackageJson = JSON.parse(readFileSync(packageJsonPath, 'utf-8'));
const newVersion = updatedPackageJson.version;

console.log(`\nNew version: ${newVersion}`);

// Update Cargo.toml version
const cargoTomlPath = join(process.cwd(), 'src-tauri', 'Cargo.toml');
let cargoToml = readFileSync(cargoTomlPath, 'utf-8');
cargoToml = cargoToml.replace(/^version\s*=\s*"[^"]+"/m, `version = "${newVersion}"`);
writeFileSync(cargoTomlPath, cargoToml);

// Update tauri.conf.json version
const tauriConfPath = join(process.cwd(), 'src-tauri', 'tauri.conf.json');
let tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));
tauriConf.version = newVersion;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');

// Commit version bump
exec('git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json');
exec(`git commit -m "chore: bump version to ${newVersion}"`);

// Create tag
exec(`git tag v${newVersion}`);

// Push
exec('git push origin main');
exec(`git push origin v${newVersion}`);

console.log('\n✅ Release created successfully!');
console.log(`Version: v${newVersion}`);
console.log('CI will now build and publish artifacts automatically.');
console.log('\nCheck the progress at:');
console.log('https://github.com/DeltaWhiplash/Timetable/actions');
