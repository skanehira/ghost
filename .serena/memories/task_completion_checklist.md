# Task Completion Checklist

When completing any coding task in the Ghost project, ensure you:

## 1. Code Quality Checks
- [ ] Run `cargo fmt` to format code
- [ ] Run `cargo clippy -- -D warnings` to check for linting issues
- [ ] Run `cargo test` to ensure all tests pass
- [ ] Run `cargo build` to verify compilation

## 2. Test Coverage
- [ ] Add unit tests for new functionality
- [ ] Update existing tests if behavior changes
- [ ] Run `cargo nextest run` for better test output
- [ ] Consider adding integration tests for complex features

## 3. Documentation
- [ ] Update inline documentation (/// comments) for public APIs
- [ ] Update README.md if user-facing features change
- [ ] Update CLAUDE.md if development process changes

## 4. Git Hygiene
- [ ] Make atomic commits with clear messages
- [ ] Separate structural changes from behavioral changes
- [ ] Run `git diff` to review changes before committing

## 5. CI/CD Verification
- [ ] Ensure GitHub Actions will pass:
  - Format check: `cargo fmt --all -- --check`
  - Clippy: `cargo clippy -- -D warnings`
  - Tests on all platforms (Linux, macOS, Windows)
  - Benchmarks don't regress significantly

## 6. Platform Compatibility
- [ ] Test on macOS if making platform-specific changes
- [ ] Ensure Unix-specific code is properly gated
- [ ] Check that file paths work across platforms

## Common Commands to Run Before Marking Task Complete:
```bash
# Full verification suite
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build --release
```