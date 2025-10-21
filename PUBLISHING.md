# Publishing the Vortex Rust SDK to crates.io

This guide walks you through publishing the Vortex Rust SDK to crates.io so users can install it with `cargo add vortex-sdk`.

## Overview

[crates.io](https://crates.io) is the official Rust package registry. Publishing is straightforward and requires:
- A crates.io account (linked to GitHub)
- Rust and Cargo installed
- A properly configured `Cargo.toml` file
- An API token from crates.io

## Prerequisites

### 1. Install Rust

If you don't have Rust installed:

```bash
# Install Rust using rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then restart your shell or run:
source $HOME/.cargo/env

# Verify installation
cargo --version
rustc --version
```

### 2. Create crates.io Account

1. Go to [crates.io](https://crates.io)
2. Click "Log in with GitHub" in the top right
3. Authorize the application

### 3. Get API Token

1. Go to [crates.io/settings/tokens](https://crates.io/me)
2. Click "New Token"
3. Give it a name (e.g., "vortex-sdk-publishing")
4. Copy the token (you won't see it again!)

### 4. Configure Cargo Credentials

```bash
# Save your API token
cargo login YOUR_API_TOKEN_HERE
```

This saves the token to `~/.cargo/credentials.toml`

## Publishing Process

### Step 1: Verify Cargo.toml Configuration

The `Cargo.toml` file has been configured with:
- ✅ Package name, version, authors
- ✅ Description and documentation
- ✅ Repository and homepage URLs
- ✅ License (MIT)
- ✅ Keywords and categories for discoverability
- ✅ Dependencies
- ✅ Rust edition (2021)

Current Cargo.toml: [Cargo.toml](Cargo.toml)

**Important**: The crate name on crates.io will be `vortex-sdk` (not `vortex-rust-sdk`)

### Step 2: Update Version

Edit [Cargo.toml](Cargo.toml) version field:

```toml
[package]
version = "1.0.0"
```

### Step 3: Create CHANGELOG

Create or update `CHANGELOG.md` with release notes:

```markdown
# Changelog

## [1.0.0] - 2025-01-20

### Added
- Initial release
- JWT generation for Vortex Widget authentication
- Async API client for Vortex API
- Invitation management (get, revoke, accept)
- Group operations
- Type-safe error handling
- Full tokio async/await support
```

### Step 4: Build and Test

```bash
cd packages/vortex-rust-sdk

# Check for any issues
cargo check

# Run tests
cargo test

# Build in release mode
cargo build --release

# Check formatting
cargo fmt --check

# Run Clippy (Rust linter)
cargo clippy -- -D warnings

# Build documentation locally to verify
cargo doc --open
```

### Step 5: Verify Package Contents

```bash
# See what will be included in the published package
cargo package --list

# Do a dry-run of publishing (doesn't actually publish)
cargo publish --dry-run
```

This creates a `.crate` file in `target/package/` that you can inspect.

### Step 6: Publish to crates.io

```bash
# Publish the crate
cargo publish
```

That's it! The crate will be published to crates.io.

### Step 7: Verify Publication

After publishing:

1. Check [crates.io/crates/vortex-sdk](https://crates.io/crates/vortex-sdk)
2. Wait a few minutes for documentation to build at [docs.rs/vortex-sdk](https://docs.rs/vortex-sdk)
3. Test installation:
   ```bash
   cargo new test-vortex
   cd test-vortex
   cargo add vortex-sdk
   ```

## Installation for Users

Once published, users can install with:

```bash
cargo add vortex-sdk
```

Or manually add to `Cargo.toml`:

```toml
[dependencies]
vortex-sdk = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Automated Publishing with GitHub Actions

Create `.github/workflows/publish-rust.yml`:

```yaml
name: Publish Rust Crate

on:
  release:
    types: [published]
  workflow_dispatch:

jobs:
  publish:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: |
          cd packages/vortex-rust-sdk
          cargo test --all-features

      - name: Run Clippy
        run: |
          cd packages/vortex-rust-sdk
          cargo clippy -- -D warnings

      - name: Check formatting
        run: |
          cd packages/vortex-rust-sdk
          cargo fmt --check

      - name: Publish to crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cd packages/vortex-rust-sdk
          cargo publish --token $CARGO_REGISTRY_TOKEN
```

### GitHub Secrets Setup

Add this secret to your repository:
- `CARGO_REGISTRY_TOKEN` - Your crates.io API token

## Version Management

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0 → 2.0.0): Breaking API changes
- **MINOR** (1.0.0 → 1.1.0): New features, backward compatible
- **PATCH** (1.0.0 → 1.0.1): Bug fixes, backward compatible

### Pre-release Versions

For alpha, beta, or RC releases:

```toml
version = "1.0.0-alpha.1"
version = "1.0.0-beta.1"
version = "1.0.0-rc.1"
```

Users install with:
```bash
cargo add vortex-sdk@1.0.0-beta.1
```

## Cargo.toml Best Practices

### Required Fields

- ✅ `name` - Crate name (must be unique on crates.io)
- ✅ `version` - Current version
- ✅ `edition` - Rust edition (2021)
- ✅ `authors` - Author names
- ✅ `description` - Package description
- ✅ `license` - License identifier (e.g., "MIT")

### Recommended Fields

```toml
[package]
repository = "https://github.com/teamvortexsoftware/vortex-rust-sdk"
homepage = "https://vortexsoftware.com"
documentation = "https://docs.rs/vortex-sdk"
keywords = ["vortex", "authentication", "invitations", "jwt"]
categories = ["authentication", "api-bindings"]
readme = "README.md"
```

### Dependencies

Be specific about version requirements:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
```

### Include/Exclude Files

Control what gets published:

```toml
[package]
exclude = [
    "target/",
    ".git/",
    ".github/",
    "examples/",
    "tests/integration/",
]
```

Or use `include` instead:

```toml
[package]
include = [
    "src/**/*.rs",
    "Cargo.toml",
    "README.md",
    "LICENSE",
]
```

## Managing Crate Ownership

### Add Collaborators

```bash
# Add a collaborator by GitHub username
cargo owner --add github_username vortex-sdk
```

### Remove Collaborators

```bash
cargo owner --remove github_username vortex-sdk
```

### List Owners

```bash
cargo owner --list vortex-sdk
```

## Yanking a Release

If you need to prevent new projects from using a version:

```bash
# Yank a version
cargo yank --vers 1.0.0 vortex-sdk

# Un-yank if you change your mind
cargo yank --undo --vers 1.0.0 vortex-sdk
```

**Note**: Yanking doesn't delete the crate or break existing projects. It just prevents new projects from depending on that version.

## Documentation

### Doc Comments

Rust uses doc comments that become documentation:

```rust
/// Generates a JWT for authenticating with the Vortex Widget
///
/// # Arguments
///
/// * `user_id` - The user's unique identifier in your system
/// * `identifiers` - Contact methods for the user (email, SMS, etc.)
/// * `groups` - Groups the user belongs to
/// * `role` - Optional role for the user
///
/// # Example
///
/// ```
/// use vortex_sdk::{VortexClient, Identifier, Group};
///
/// let client = VortexClient::new("your-api-key");
/// let jwt = client.generate_jwt(
///     "user-123",
///     vec![Identifier::new("email", "user@example.com")],
///     vec![Group::new("workspace", "ws-1", "Main")],
///     Some("admin"),
/// )?;
/// ```
pub fn generate_jwt(...) -> Result<String, VortexError> {
    // implementation
}
```

### Generate Documentation

```bash
# Build and open documentation locally
cargo doc --open

# Build docs with all features
cargo doc --all-features --no-deps
```

### docs.rs

Published crates automatically get documentation at [docs.rs](https://docs.rs). Your crate will be at:
- https://docs.rs/vortex-sdk

## Testing Before Release

### Comprehensive Testing

```bash
cd packages/vortex-rust-sdk

# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run tests in release mode (catches optimization issues)
cargo test --release

# Run doc tests
cargo test --doc

# Build examples
cargo build --examples

# Run a specific example
cargo run --example basic_usage
```

### Local Installation Test

```bash
# Install from local path
cd /tmp
cargo new test-vortex
cd test-vortex

# Add local dependency
cargo add vortex-sdk --path /path/to/packages/vortex-rust-sdk

# Create a test
cat > src/main.rs << 'EOF'
use vortex_sdk::{VortexClient, Identifier, Group};

fn main() {
    let client = VortexClient::new("test-key".to_string());
    let jwt = client.generate_jwt(
        "user-123",
        vec![Identifier::new("email", "test@example.com")],
        vec![Group::new("team", "team-1", "Test Team")],
        Some("member"),
    );
    match jwt {
        Ok(token) => println!("JWT generated: {}", token.chars().take(20).collect::<String>()),
        Err(e) => println!("Error: {}", e),
    }
}
EOF

# Run it
cargo run
```

## Troubleshooting

### Common Issues

#### 1. Crate Name Already Taken

Check if the name is available:

```bash
cargo search vortex-sdk --limit 5
```

If taken, you'll need to choose a different name. Consider:
- `vortex-client`
- `vortex-rs`
- `vortex-auth-sdk`

Update the name in `Cargo.toml`:
```toml
[package]
name = "your-chosen-name"
```

#### 2. Authentication Errors

```bash
# Re-login to crates.io
cargo login YOUR_NEW_TOKEN

# Check credentials
cat ~/.cargo/credentials.toml
```

#### 3. Version Already Published

You cannot publish the same version twice. Update the version in `Cargo.toml`:

```toml
[package]
version = "1.0.1"
```

Then rebuild and publish.

#### 4. Missing Dependencies

If you get missing dependency errors:

```bash
# Update dependencies
cargo update

# Or specify exact versions in Cargo.toml
```

#### 5. Documentation Build Failures

If docs.rs fails to build:

```bash
# Test documentation build locally
cargo doc --no-deps

# Check for broken links
cargo rustdoc -- -D warnings
```

#### 6. File Size Limit

crates.io has a 10 MB limit. If your package is too large:

```bash
# Check package size
cargo package --list

# Exclude large files in Cargo.toml
[package]
exclude = ["target/", "*.png", "docs/"]
```

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Update `README.md` if needed
- [ ] Run all tests: `cargo test --all-features`
- [ ] Run Clippy: `cargo clippy -- -D warnings`
- [ ] Check formatting: `cargo fmt --check`
- [ ] Build docs: `cargo doc --all-features`
- [ ] Dry-run publish: `cargo publish --dry-run`
- [ ] Commit version bump: `git commit -am "Bump version to X.Y.Z"`
- [ ] Create Git tag: `git tag vX.Y.Z`
- [ ] Push changes: `git push && git push --tags`
- [ ] Publish crate: `cargo publish`
- [ ] Verify on crates.io
- [ ] Wait for docs.rs to build
- [ ] Test installation: `cargo add vortex-sdk@X.Y.Z`
- [ ] Create GitHub release with changelog
- [ ] Announce release

## Security Best Practices

1. **Protect API tokens**: Never commit `~/.cargo/credentials.toml` to Git
2. **Use GitHub Secrets**: For automated publishing
3. **Minimal permissions**: API tokens should only have publish permissions
4. **Review dependencies**: Regularly audit with `cargo audit`
5. **Dependabot**: Enable on GitHub for dependency updates

## Post-Publication

After publishing:

1. **Verify installation**: Test on a clean system
2. **Monitor downloads**: Check stats on crates.io
3. **Check docs.rs**: Ensure documentation built successfully
4. **Watch for issues**: Monitor GitHub issues
5. **Keep dependencies updated**: Run `cargo update` regularly
6. **Test with latest Rust**: Test with `rustc --version` updates

## Useful Commands

```bash
# Search for your crate
cargo search vortex-sdk

# Get crate info
cargo info vortex-sdk

# List crate owners
cargo owner --list vortex-sdk

# Yank a version
cargo yank --vers 1.0.0 vortex-sdk

# Generate docs
cargo doc --open

# Run Clippy
cargo clippy

# Format code
cargo fmt

# Check without building
cargo check

# Build optimized
cargo build --release

# Run benchmarks (if you add them)
cargo bench

# Update dependencies
cargo update

# Audit dependencies for security issues
cargo install cargo-audit
cargo audit
```

## Resources

- [crates.io](https://crates.io) - Rust package registry
- [The Cargo Book](https://doc.rust-lang.org/cargo/) - Official Cargo documentation
- [Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Cargo.toml Reference](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [docs.rs](https://docs.rs) - Automatic documentation hosting
- [Semantic Versioning](https://semver.org/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Crate Naming Conventions

- Use lowercase with hyphens: `vortex-sdk` (not `VortexSDK` or `vortex_sdk`)
- Keep it concise and memorable
- Avoid redundant `-rs` or `-rust` suffixes unless necessary
- Make it searchable and discoverable

## Advanced: Features and Feature Flags

If you want to add optional features:

```toml
[features]
default = ["tokio-runtime"]
tokio-runtime = ["tokio"]
async-std-runtime = ["async-std"]
```

Users can then opt in/out:

```bash
cargo add vortex-sdk --no-default-features --features async-std-runtime
```

## Support

For publishing issues:
- [crates.io Help](https://crates.io/help)
- [Rust Users Forum](https://users.rust-lang.org/)
- [Rust Discord](https://discord.gg/rust-lang)

For SDK issues:
- Create an issue on GitHub
- Contact support@vortexsoftware.com
