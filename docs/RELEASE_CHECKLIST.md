# Release Checklist

This checklist ensures consistent, high-quality releases of Macaw using cargo-dist for automated binary builds.

## Pre-Release

### Code Quality

- [ ] All tests pass: `just check`
- [ ] No security vulnerabilities: `cargo audit`
- [ ] Code is formatted: `cargo fmt --check`
- [ ] Linter is happy: `cargo clippy`
- [ ] Documentation is up to date

### Version Updates

- [ ] Update version in `Cargo.toml` (follow [Semantic Versioning](https://semver.org/))
- [ ] Rebuild to update `Cargo.lock`: `cargo build`
- [ ] Update CHANGELOG.md (if maintained)
- [ ] Commit version changes: `git commit -am "Bump version to vX.Y.Z"`
- [ ] Push changes to main branch

### Pre-Release Testing

- [ ] Test local build: `just dist_build`
- [ ] Verify artifacts in `target/distrib/`
- [ ] Test binary on local platform: `./target/distrib/macaw-*/macaw --version`
- [ ] Review what will be released: `just dist_plan`

## Release

### Create Release

- [ ] Ensure you're on main branch: `git checkout main && git pull`
- [ ] Create release: `just release vX.Y.Z`
- [ ] Monitor GitHub Actions: <https://github.com/fini-net/macaw/actions>
- [ ] Wait for all platform builds to complete (~15-20 minutes)

### Verify Release Artifacts

- [ ] Check GitHub release page: <https://github.com/fini-net/macaw/releases>
- [ ] Verify all 4 platform tarballs are present:
  - [ ] `macaw-vX.Y.Z-aarch64-apple-darwin.tar.gz`
  - [ ] `macaw-vX.Y.Z-x86_64-apple-darwin.tar.gz`
  - [ ] `macaw-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
  - [ ] `macaw-vX.Y.Z-x86_64-unknown-linux-musl.tar.gz`
- [ ] Verify SHA256 checksums file is attached
- [ ] Verify shell installer is present: `macaw-installer.sh`
- [ ] Check release notes are auto-generated

## Post-Release

### Testing

- [ ] Download and test shell installer:

  ```bash
  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/fini-net/macaw/releases/download/vX.Y.Z/macaw-installer.sh | sh
  ```

- [ ] Test binary works: `macaw --version`
- [ ] Verify version matches release

### Cleanup

- [ ] Delete local test artifacts: `rm -rf target/distrib/`
- [ ] Tag is pushed and matches release
- [ ] Main branch is up to date: `git pull`

## Communication

### Internal

- [ ] Notify team of new release (if applicable)
- [ ] Update deployment documentation if needed

### External (when ready for public releases)

- [ ] Announce on project website/blog
- [ ] Post to relevant social media
- [ ] Update package managers (if applicable)
- [ ] Notify users of breaking changes (if any)

## Rollback Plan

If critical issues are discovered post-release:

1. **Delete the release:**

   ```bash
   gh release delete vX.Y.Z --yes
   ```

2. **Delete the tag:**

   ```bash
   git tag -d vX.Y.Z
   git push origin :refs/tags/vX.Y.Z
   ```

3. **Fix the issue:**
   - Create a hotfix branch
   - Apply fixes
   - Run through this checklist again with a new version

4. **Create a new release with incremented version**

## Notes

- cargo-dist builds are deterministic and reproducible
- All builds run in GitHub Actions (no local builds in CI)
- Binaries are optimized with LTO (Link-Time Optimization)
- Each release should have a corresponding git tag
- Never force push to main or modify tags after release
- Prereleases are not published to Homebrew (per `publish-prereleases = false`)

## Version Numbering Guide

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Incompatible API changes
- **MINOR** (0.X.0): New functionality, backwards compatible
- **PATCH** (0.0.X): Backwards compatible bug fixes

Pre-release versions (optional):

- **Alpha**: `vX.Y.Z-alpha.1` - Early testing, unstable
- **Beta**: `vX.Y.Z-beta.1` - Feature complete, needs testing
- **RC**: `vX.Y.Z-rc.1` - Release candidate, final testing

## References

- [cargo-dist Documentation](https://opensource.axo.dev/cargo-dist/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
