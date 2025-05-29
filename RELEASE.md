# Multi-Architecture Release Guide

This guide explains how to build and release CodeRAG for multiple architectures.

## Supported Platforms

| Platform | Architecture | Binary Name | Notes |
|----------|-------------|------------|-------|
| Linux | x86_64 | `coderag-mcp-linux-amd64` | Most common Linux servers |
| Linux | aarch64 | `coderag-mcp-linux-arm64` | ARM64 servers, Raspberry Pi 4 |
| macOS | x86_64 | `coderag-mcp-macos-amd64` | Intel Macs |
| macOS | aarch64 | `coderag-mcp-macos-arm64` | Apple Silicon (M1/M2/M3) |
| macOS | Universal | `coderag-mcp-macos-universal` | Works on both Intel and Apple Silicon |
| Windows | x86_64 | `coderag-mcp-windows-amd64.exe` | 64-bit Windows |
| Windows | aarch64 | `coderag-mcp-windows-arm64.exe` | ARM64 Windows devices |

## Automated Releases (GitHub Actions)

Releases are automatically built when you push a tag starting with `v`:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Action will:
1. Create a new release
2. Build binaries for all supported platforms
3. Upload the binaries as release assets
4. Generate SHA256 checksums

## Local Building

### Prerequisites

1. Install Rust targets:
```bash
task install-targets
```

2. For Linux cross-compilation on macOS, you'll need:
   - Docker Desktop (for using cross-rs)
   - Or install the GNU toolchain via Homebrew

### Build Commands

Build for your current platform:
```bash
task release
```

Build for all platforms (requires cross-compilation setup):
```bash
task release-all
```

Build for specific platforms:
```bash
# Linux
task release-linux-amd64
task release-linux-arm64

# macOS
task release-macos-amd64
task release-macos-arm64
task release-macos-universal

# Windows (requires Windows or cross-compilation)
task release-windows-amd64
```

### Cross-Compilation Setup

#### Linux targets on macOS/Windows

Install and use `cross`:
```bash
cargo install cross
```

Then modify the Taskfile commands to use `cross` instead of `cargo`:
```bash
cross build --release --bin coderag-mcp --target aarch64-unknown-linux-gnu
```

#### macOS Universal Binary

On macOS, you can create a universal binary that works on both Intel and Apple Silicon:
```bash
task release-macos-universal
```

## Binary Distribution

### File Naming Convention

- Linux: `coderag-mcp-linux-{arch}`
- macOS: `coderag-mcp-macos-{arch}`
- Windows: `coderag-mcp-windows-{arch}.exe`

Where `{arch}` is:
- `amd64` for x86_64
- `arm64` for aarch64
- `universal` for macOS universal binaries

### Archive Format

- Linux/macOS: `.tar.gz` with binary + README + LICENSE
- Windows: `.zip` with binary + README + LICENSE

## Testing Cross-Compiled Binaries

1. Use Docker to test Linux binaries:
```bash
# Test Linux AMD64
docker run --rm -v $(pwd):/app ubuntu:latest /app/coderag-mcp-linux-amd64 --version

# Test Linux ARM64 (on Apple Silicon or with QEMU)
docker run --rm -v $(pwd):/app --platform linux/arm64 ubuntu:latest /app/coderag-mcp-linux-arm64 --version
```

2. Test macOS binaries natively or in VMs

3. Test Windows binaries in Windows VMs or on actual hardware

## Troubleshooting

### Linux Cross-Compilation Issues

If you encounter linking errors, use Docker-based cross-compilation:
```bash
docker run --rm -v $(pwd):/workspace \
  -w /workspace \
  messense/rust-musl-cross:x86_64-musl \
  cargo build --release --bin coderag-mcp --target x86_64-unknown-linux-musl
```

### macOS Code Signing

For distribution outside the App Store, you may need to sign your binaries:
```bash
codesign -s "Developer ID Application: Your Name" coderag-mcp-macos-*
```

### Windows Dependencies

Windows binaries require the Visual C++ Redistributable. Consider using static linking:
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

## Version Management

Update version in:
1. `Cargo.toml` - package version
2. `Taskfile.yml` - HF_HUB_USER_AGENT_ORIGIN
3. Tag the release in git

## Release Checklist

- [ ] Update version numbers
- [ ] Run tests on all platforms
- [ ] Update CHANGELOG.md
- [ ] Create git tag
- [ ] Push tag to trigger automated build
- [ ] Verify all platform binaries work
- [ ] Update installation documentation
