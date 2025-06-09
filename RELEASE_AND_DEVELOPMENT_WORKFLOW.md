# Release and Development Workflow for Zed Helix Mode Fork

## Current State Analysis

### Existing Infrastructure

This fork of Zed has a comprehensive CI/CD system inherited from the upstream project:

**Build and Packaging:**
- Multi-platform builds (macOS, Linux, FreeBSD, Windows)
- Multiple distribution formats (DMG, tar.gz, Flatpak, Snap)
- Code signing and notarization for macOS
- Docker-based cross-compilation
- Automated nightly builds via GitHub Actions

**Release Management:**
- Version bumping scripts for major, minor, and patch releases
- GitHub draft release creation
- Automated upload to blob storage (DigitalOcean Spaces)
- Release channel system (stable, preview, nightly, dev)

**Quality Assurance:**
- Comprehensive CI/CD pipelines
- Randomized testing
- License compliance checking
- Code linting and formatting validation

### Current Branch Structure

- `main` - Syncs with upstream Zed project
- `helix-mode` - Current development branch with Helix functionality
- `helix-mode-reuse-vim-minimal` - Alternative implementation branch

## Proposed Branching Strategy

### Branch Purpose and Workflow

```
upstream/main (Zed official)
    ↓ (sync periodically)
main (sync branch)
    ↓ (merge stable features)
helix-mode (production/stable)
    ↓ (development work)
helix-mode-dev (active development)
    ↓ (feature branches)
feature/* branches
```

### Branch Definitions

1. **`main`**
   - **Purpose:** Sync with upstream Zed project
   - **Updates:** Periodic merges from upstream Zed
   - **Direct commits:** None (only merges from upstream)
   - **Protection:** Protected branch, no direct pushes

2. **`helix-mode`** (Production Branch)
   - **Purpose:** Stable release branch with Helix functionality
   - **Updates:** Only from `helix-mode-dev` after thorough testing
   - **Releases:** All releases are created from this branch
   - **Protection:** Protected branch, requires PR reviews
   - **Merge policy:** Squash and merge from `helix-mode-dev`

3. **`helix-mode-dev`** (Development Branch)
   - **Purpose:** Active development of Helix features
   - **Updates:** Feature branches merge here first
   - **Testing:** Continuous integration and testing
   - **Stability:** May be unstable, used for development builds

4. **`feature/*`** branches
   - **Purpose:** Individual feature development
   - **Naming:** `feature/description-of-feature`
   - **Lifecycle:** Created from `helix-mode-dev`, merged back when complete
   - **Cleanup:** Deleted after successful merge

## Release Workflow

### Version Numbering

Follow semantic versioning with Helix mode suffix:
- Format: `v{major}.{minor}.{patch}-helix.{build}`
- Example: `v0.190.0-helix.1`

### Release Types

1. **Major Releases** (`v0.X.0-helix.1`)
   - Significant new Helix functionality
   - Breaking changes to Helix mode
   - Sync with major upstream Zed releases

2. **Minor Releases** (`v0.X.Y-helix.1`)
   - New Helix features
   - Non-breaking improvements
   - Bug fixes and stability improvements

3. **Patch Releases** (`v0.X.Y-helix.Z`)
   - Critical bug fixes
   - Security updates
   - Hot fixes for production issues

### Release Process

#### 1. Pre-Release Preparation
```bash
# Ensure helix-mode-dev is up to date and tested
git checkout helix-mode-dev
git pull origin helix-mode-dev

# Run comprehensive tests
./script/clippy
cargo test --workspace
```

#### 2. Create Release PR
```bash
# Create release branch from helix-mode-dev
git checkout -b release/v0.X.Y-helix.Z helix-mode-dev

# Update version numbers
# - Update Cargo.toml version
# - Update RELEASE_CHANNEL if needed
# - Update CHANGELOG.md with release notes

# Commit version bump
git add .
git commit -m "Bump version to v0.X.Y-helix.Z"

# Push and create PR to helix-mode
git push origin release/v0.X.Y-helix.Z
# Create PR: release/v0.X.Y-helix.Z → helix-mode
```

#### 3. Release Review and Testing
- **Code Review:** Minimum 2 reviewer approval
- **Testing:** Run full test suite on multiple platforms
- **Documentation:** Ensure CHANGELOG.md is updated
- **Build Verification:** Test builds on all supported platforms

#### 4. Release Execution
```bash
# Merge release PR to helix-mode
git checkout helix-mode
git merge --no-ff release/v0.X.Y-helix.Z

# Tag the release
git tag -a v0.X.Y-helix.Z -m "Release v0.X.Y-helix.Z"

# Push tag to trigger release automation
git push origin helix-mode
git push origin v0.X.Y-helix.Z
```

#### 5. Post-Release
```bash
# Merge back to development branch
git checkout helix-mode-dev
git merge helix-mode

# Clean up release branch
git branch -d release/v0.X.Y-helix.Z
git push origin --delete release/v0.X.Y-helix.Z
```

## Development Workflow

### Feature Development

1. **Create Feature Branch**
```bash
git checkout helix-mode-dev
git pull origin helix-mode-dev
git checkout -b feature/new-helix-command
```

2. **Development Process**
```bash
# Make changes and commit regularly
git add .
git commit -m "Implement new Helix command: [description]"

# Push feature branch
git push origin feature/new-helix-command
```

3. **Create Pull Request**
- **Target:** `helix-mode-dev`
- **Review:** Require at least 1 reviewer
- **Testing:** CI must pass
- **Documentation:** Update relevant docs

4. **Merge and Cleanup**
```bash
# After PR approval and merge
git checkout helix-mode-dev
git pull origin helix-mode-dev
git branch -d feature/new-helix-command
git push origin --delete feature/new-helix-command
```

### Syncing with Upstream

Periodic sync with upstream Zed (monthly or as needed):

```bash
# Add upstream remote if not exists
git remote add upstream https://github.com/zed-industries/zed.git

# Fetch upstream changes
git fetch upstream

# Update main branch
git checkout main
git merge upstream/main
git push origin main

# Merge relevant changes to development
git checkout helix-mode-dev
git merge main  # Resolve conflicts as needed
git push origin helix-mode-dev
```

## Build and Distribution

### Local Development Builds
```bash
# Debug build
cargo build

# Release build (local testing)
cargo build --release

# Platform-specific bundle
./script/bundle-mac     # macOS
./script/bundle-linux   # Linux
```

### CI/CD Integration

**Automated Builds:**
- Triggered on pushes to `helix-mode` and `helix-mode-dev`
- Build all supported platforms
- Run comprehensive test suite
- Generate build artifacts

**Release Automation:**
- Triggered by version tags on `helix-mode`
- Create GitHub releases
- Upload platform-specific binaries
- Generate release notes from CHANGELOG.md

### Distribution Channels

1. **GitHub Releases**
   - Primary distribution method
   - Platform-specific binaries
   - Source code archives

2. **Package Managers** (Future)
   - Homebrew (macOS)
   - AUR (Arch Linux)
   - Flatpak (Linux)
   - Snap (Linux)

## Configuration Files

### Release Channel Configuration
Update `crates/zed/RELEASE_CHANNEL` to distinguish from upstream:
```
helix-dev
```

### GitHub Actions Modifications
Modify `.github/workflows/` to:
- Use custom release channels
- Upload to fork-specific storage
- Tag releases with Helix suffix

## Documentation and Communication

### Release Notes
Maintain `CHANGELOG.md` with:
- New Helix features
- Bug fixes
- Breaking changes
- Migration guides

### User Communication
- GitHub Releases with detailed notes
- Documentation updates
- Community forum announcements

## Migration Plan

### Immediate Actions (Week 1)

1. **Create Development Branch**
```bash
git checkout -b helix-mode-dev helix-mode
git push origin helix-mode-dev
```

2. **Update Release Channel**
```bash
echo "helix-dev" > crates/zed/RELEASE_CHANNEL
git add crates/zed/RELEASE_CHANNEL
git commit -m "Set release channel to helix-dev"
```

3. **Protect Branches**
- Set `helix-mode` as protected branch
- Require PR reviews for `helix-mode`
- Set up branch protection rules

4. **Update CI Configuration**
- Modify release scripts to use helix-specific naming
- Update storage destinations to avoid upstream conflicts

### Future Enhancements

1. **Custom Update System**
   - Implement fork-specific auto-update mechanism
   - Use separate update servers/endpoints

2. **Branding Updates**
   - Custom application icons
   - Helix-specific UI elements
   - About dialog updates

3. **Community Infrastructure**
   - Documentation website
   - Issue templates specific to Helix mode
   - Contributing guidelines

## Risk Mitigation

### Backup Strategy
- Regular backups of release artifacts
- Git repository mirrors
- Documentation preservation

### Conflict Resolution
- Maintain clear merge conflict resolution procedures
- Document breaking changes from upstream
- Test compatibility thoroughly

### Security Considerations
- Code signing certificates management
- Secure build environment
- Dependency security scanning

This workflow provides a robust foundation for maintaining a stable Helix mode fork while enabling continued development and upstream synchronization.