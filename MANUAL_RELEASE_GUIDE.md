# Manual Release Guide for Zed Helix Mode Fork

## Prerequisites

Before starting a manual release, ensure you have:

1. **Development Environment Setup**
   ```bash
   # Install dependencies
   ./script/bootstrap
   
   # Verify build environment
   cargo --version
   rustc --version
   ```

2. **Permissions and Access**
   - Write access to the repository
   - Code signing certificates (for macOS)
   - Access to storage/distribution endpoints (if configured)

3. **Clean Working Directory**
   ```bash
   git status  # Should show clean working tree
   git fetch --all
   ```

## Step-by-Step Manual Release Process

### Phase 1: Pre-Release Preparation

#### 1.1 Verify Current State
```bash
# Check current branch and status
git checkout helix-mode
git pull origin helix-mode
git status

# Verify the codebase builds and tests pass
./script/clippy
cargo test --workspace
cargo build --release
```

#### 1.2 Determine Version Number
Follow semantic versioning with Helix suffix:
- Major: `v0.X.0-helix.1` (significant changes)
- Minor: `v0.X.Y-helix.1` (new features)
- Patch: `v0.X.Y-helix.Z` (bug fixes)

#### 1.3 Update Version Information
```bash
# Update main Cargo.toml
vim Cargo.toml
# Change version = "0.X.Y" to your target version

# Update release channel if needed
echo "helix-stable" > crates/zed/RELEASE_CHANNEL

# Update changelog
vim CHANGELOG.md
# Add release notes for this version
```

### Phase 2: Version Bump and Tagging

#### 2.1 Commit Version Changes
```bash
# Stage version changes
git add Cargo.toml crates/zed/RELEASE_CHANNEL CHANGELOG.md

# Commit with conventional format
git commit -m "release: bump version to v0.X.Y-helix.Z

- Update Cargo.toml version
- Set release channel to helix-stable
- Add changelog entries for v0.X.Y-helix.Z"
```

#### 2.2 Create and Push Tag
```bash
# Create annotated tag
git tag -a v0.X.Y-helix.Z -m "Release v0.X.Y-helix.Z

New features:
- [List key features]

Bug fixes:
- [List major fixes]

See CHANGELOG.md for complete details."

# Push commit and tag
git push origin helix-mode
git push origin v0.X.Y-helix.Z
```

### Phase 3: Manual Build Process

If you need to build manually instead of using CI/CD:

#### 3.1 macOS Build
```bash
# Build universal binary for macOS
./script/bundle-mac

# This creates:
# - target/release/Zed.dmg
# - Debug symbols uploaded to blob storage (if configured)
```

#### 3.2 Linux Build
```bash
# Build Linux tarball
./script/bundle-linux

# This creates:
# - target/release/zed-linux-x86_64.tar.gz
# - Debug symbols uploaded separately
```

#### 3.3 Alternative: Docker-based Cross-compilation
```bash
# Build for multiple platforms using Docker
./script/build-docker

# Or build specific platform
docker run --rm -v $(pwd):/workspace -w /workspace \
  rust:latest cargo build --release --target x86_64-unknown-linux-gnu
```

### Phase 4: Release Creation

#### 4.1 Create GitHub Release
```bash
# Using GitHub CLI (if available)
gh release create v0.X.Y-helix.Z \
  --title "Zed Helix Mode v0.X.Y-helix.Z" \
  --notes-file CHANGELOG.md \
  --draft

# Or create manually via GitHub web interface
```

#### 4.2 Upload Build Artifacts
```bash
# Upload to GitHub release (if using gh CLI)
gh release upload v0.X.Y-helix.Z target/release/Zed.dmg
gh release upload v0.X.Y-helix.Z target/release/zed-linux-x86_64.tar.gz

# Or upload via web interface
```

### Phase 5: Post-Release Tasks

#### 5.1 Merge Back to Development
```bash
# Ensure helix-mode-dev gets the release changes
git checkout helix-mode-dev
git pull origin helix-mode-dev
git merge helix-mode
git push origin helix-mode-dev
```

#### 5.2 Publish Release
```bash
# Remove draft status from GitHub release
gh release edit v0.X.Y-helix.Z --draft=false

# Or publish via web interface
```

#### 5.3 Update Development Version
```bash
# Bump to next development version
git checkout helix-mode-dev

# Update version in Cargo.toml to next dev version
# e.g., if you released 0.190.0-helix.1, bump to 0.190.1-helix.0-dev
vim Cargo.toml

# Update release channel
echo "helix-dev" > crates/zed/RELEASE_CHANNEL

git add Cargo.toml crates/zed/RELEASE_CHANNEL
git commit -m "chore: bump to development version v0.X.Y-helix.Z-dev"
git push origin helix-mode-dev
```

## Emergency Hotfix Release

For critical bug fixes that need immediate release:

### 1. Create Hotfix Branch
```bash
git checkout helix-mode
git checkout -b hotfix/v0.X.Y-helix.Z+1
```

### 2. Apply Minimal Fix
```bash
# Make only the essential changes
git add .
git commit -m "hotfix: fix critical issue [description]"
```

### 3. Test Thoroughly
```bash
./script/clippy
cargo test --workspace
cargo build --release
# Manual testing of the fix
```

### 4. Fast-Track Release
```bash
# Update version (patch increment)
# Follow normal release process but expedited review
```

## Manual Build Commands Reference

### Development Build
```bash
cargo build                    # Debug build
cargo build --release         # Release build
```

### Platform-Specific Builds
```bash
./script/bundle-mac           # macOS DMG
./script/bundle-linux         # Linux tarball  
./script/bundle-freebsd       # FreeBSD package
```

### Package Formats
```bash
./script/flatpak/bundle-flatpak  # Flatpak package
./script/snap-build              # Snap package
```

### Quality Checks
```bash
./script/clippy               # Linting
cargo test --workspace       # All tests
./script/check-licenses       # License compliance
```

## Troubleshooting Common Issues

### Build Failures
```bash
# Clean build artifacts
cargo clean
rm -rf target/

# Update Rust toolchain
rustup update

# Check for platform-specific dependencies
./script/bootstrap
```

### Signing Issues (macOS)
```bash
# Verify certificates
security find-identity -v -p codesigning

# Check entitlements
codesign -d --entitlements - target/release/Zed.app
```

### Version Conflicts
```bash
# Check current version
cargo metadata --format-version=1 | jq '.packages[] | select(.name=="zed") | .version'

# Verify tag doesn't exist
git tag -l "v0.X.Y-helix.Z"
```

## Rollback Procedure

If you need to rollback a release:

### 1. Delete Tag and Release
```bash
# Delete remote tag
git push origin --delete v0.X.Y-helix.Z

# Delete local tag
git tag -d v0.X.Y-helix.Z

# Delete GitHub release via web interface or CLI
gh release delete v0.X.Y-helix.Z
```

### 2. Revert Version Changes
```bash
git checkout helix-mode
git revert <commit-hash-of-version-bump>
git push origin helix-mode
```

## Release Checklist

**Pre-Release:**
- [ ] Clean working directory
- [ ] All tests pass
- [ ] Version numbers updated
- [ ] Changelog updated
- [ ] Build artifacts created
- [ ] Code review completed (if team process)

**Release:**
- [ ] Tag created and pushed
- [ ] GitHub release created
- [ ] Artifacts uploaded
- [ ] Release notes published

**Post-Release:**
- [ ] Development branch updated
- [ ] Next development version set
- [ ] Release announcement made (if applicable)
- [ ] Documentation updated

This manual process gives you full control over the release while leveraging the existing build infrastructure.