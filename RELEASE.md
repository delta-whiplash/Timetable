# Release Process

This document explains how to create a new release of Timetable Desktop.

## Quick Release

The easiest way to create a release is using the release script:

```bash
# For patch release (0.9.0 -> 0.9.1)
npm run release:patch

# For minor release (0.9.0 -> 0.10.0)
npm run release:minor

# For major release (0.9.0 -> 1.0.0)
npm run release:major
```

The script will automatically:
1. Check you're on the main branch with no uncommitted changes
2. Bump the version in package.json, Cargo.toml, and tauri.conf.json
3. Commit the version changes
4. Create a git tag (v0.9.1)
5. Push to GitHub
6. Trigger the CI to build multi-platform installers
7. Create a GitHub Release with all artifacts

## Manual Release

If you prefer to do it manually:

```bash
# 1. Bump version (choose one)
npm version patch  # 0.9.0 -> 0.9.1
npm version minor  # 0.9.0 -> 0.10.0
npm version major  # 0.9.0 -> 1.0.0

# 2. Update Cargo.toml and tauri.conf.json manually to match

# 3. Commit
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "chore: bump version to X.Y.Z"

# 4. Create and push tag
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

## What Happens After You Push a Tag

When you push a tag (e.g., v0.9.1), the following happens automatically:

1. **CI Tests Run** - Frontend and backend tests are executed
2. **Multi-Platform Build** - The app is built for:
   - Windows (NSIS installer + MSI installer)
   - Linux (AppImage)
   - macOS (DMG with universal binary)
3. **GitHub Release Created** - A new release is created with all installers attached

You can monitor the progress at: https://github.com/DeltaWhiplash/Timetable/actions

## Versioning Guidelines

We follow [Semantic Versioning](https://semver.org/):

- **Patch** (0.9.0 → 0.9.1): Bug fixes, performance improvements, no breaking changes
- **Minor** (0.9.0 → 0.10.0): New features, backwards compatible
- **Major** (0.9.0 → 1.0.0): Breaking changes, major refactors

## CI vs Release

- **CI** (`.github/workflows/ci.yml`): Runs on every push/PR - tests, lint, type-check
- **Release** (`.github/workflows/release.yml`): Runs only on tags - builds installers

## Release Checklist

Before creating a release:

- [ ] All tests pass locally (`npm test` and `cargo test`)
- [ ] You're on the main branch
- [ ] No uncommitted changes
- [ ] CHANGELOG.md is updated (if you maintain one)
- [ ] README.md is up to date

After creating a release:

- [ ] Check GitHub Actions for build success
- [ ] Download and test the installers
- [ ] Update release notes if needed
- [ ] Announce the release (if applicable)

## Troubleshooting

### Build fails on CI

Check the GitHub Actions logs. Common issues:
- Missing dependencies in Cargo.toml
- TypeScript errors
- Rust compilation errors
- Missing system dependencies (for Linux/macOS)

### Tag already exists

If you need to re-release the same version:
```bash
git tag -d vX.Y.Z           # Delete local tag
git push origin :refs/tags/vX.Y.Z  # Delete remote tag
# Make your fixes and create the tag again
```

### Need to cancel a release

If you pushed a bad tag:
```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
# Delete the GitHub release manually from the web interface
```

## Local Build

To test the build locally before releasing:

```bash
# Build for current platform
npm run tauri:build

# Installer will be in:
# - Windows: src-tauri/target/release/bundle/nsis/*.exe
# - Linux: src-tauri/target/release/bundle/appimage/*.AppImage
# - macOS: src-tauri/target/release/bundle/dmg/*.dmg
```

## Notes

- The release workflow has a 60-minute timeout per platform
- Artifacts are kept for 90 days by default
- The workflow uses caching to speed up builds
- Universal macOS binary supports both Intel and Apple Silicon
