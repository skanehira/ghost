# CI/CD Workflows

Ghost uses GitHub Actions for continuous integration and deployment.

## Workflows

### 1. CI (.github/workflows/ci.yaml)
- **Triggers**: Push to main, pull requests
- **Jobs**:
  - Format check: `cargo fmt --all -- --check`
  - Clippy linting: `cargo clippy -- -D warnings`
  - Build on multiple platforms (Linux, macOS, Windows)
  - Run tests with cargo-nextest
  - Generate code coverage on Linux

### 2. Audit (.github/workflows/audit.yaml)
- **Triggers**: Push, pull requests, daily schedule
- **Purpose**: Security vulnerability scanning
- **Tool**: cargo-audit

### 3. Benchmark (.github/workflows/benchmark.yaml)
- **Triggers**: Push to main
- **Purpose**: Performance regression detection
- **Tool**: cargo bench with criterion

### 4. Release (.github/workflows/release.yaml)
- **Triggers**: Git tags matching 'v*'
- **Platforms**: 
  - x86_64-unknown-linux-musl (Ubuntu)
  - aarch64-unknown-linux-musl (Ubuntu ARM64)
  - x86_64-apple-darwin (macOS)
  - aarch64-apple-darwin (macOS M1)
- **Outputs**: Compressed binaries (.tar.gz)
- **Static linking**: musl targets for Linux compatibility

## Release Process
1. Tag with version: `git tag v0.1.0`
2. Push tag: `git push origin v0.1.0`
3. GitHub Actions builds for all platforms
4. Creates GitHub Release with artifacts
5. Generates release notes automatically

## Platform-Specific Notes
- Linux builds use musl for static linking (no glibc dependency)
- macOS builds use native runners (including M1 for ARM64)
- No daemon/service files needed (simple binary distribution)