# Publishing Guide for stm32f411ve-disco

This guide walks through publishing this crate to crates.io.

## Prerequisites

### 1. Create a crates.io Account

1. Go to https://crates.io
2. Click "Log in with GitHub"
3. Authorize the application

### 2. Get Your API Token

1. Go to https://crates.io/me
2. Click "New Token"
3. Give it a name (e.g., "stm32f411ve-disco-publishing")
4. Copy the token

### 3. Configure Cargo

```bash
cargo login <your-token-here>
```

This stores your token in `~/.cargo/credentials.toml` for future use.

## Pre-Publishing Checklist

### ✅ Verify Cargo.toml Metadata

- [x] `name` - Must be unique on crates.io
- [x] `version` - Start with 0.1.0 for initial release
- [x] `authors` - Your name and email
- [x] `edition` - Using "2021"
- [x] `description` - Short description (< 200 chars)
- [x] `license` - "MIT OR Apache-2.0" (dual license)
- [x] `repository` - GitHub URL
- [x] `readme` - "README.md"
- [x] `keywords` - Max 5 keywords
- [x] `categories` - Valid crates.io categories

### ✅ Verify Files

- [x] `README.md` - Comprehensive documentation
- [x] `LICENSE-MIT` - MIT license text
- [x] `LICENSE-APACHE` - Apache 2.0 license text
- [x] All source files have proper documentation

### ✅ Test the Package

```bash
# Build the library
cargo build

# Check for issues
cargo clippy -- -D warnings

# Verify documentation builds
cargo doc --no-deps --open

# Test packaging (creates a .crate file locally)
cargo package --allow-dirty

# List files that will be included
cargo package --list
```

### ✅ Review .gitignore

Ensure `.gitignore` excludes:
- `/target/`
- `Cargo.lock` (for libraries)
- IDE files
- OS files

### ✅ Git Setup

```bash
# Ensure all changes are committed
git status

# Tag the release
git tag -a v0.1.0 -m "Initial release"

# Push to GitHub
git push origin main
git push origin v0.1.0
```

## Publishing Process

### 1. Dry Run

Test the publishing process without actually publishing:

```bash
cargo publish --dry-run
```

This will:
- Build your crate
- Package it
- Verify all metadata
- Check that documentation builds
- Show any errors WITHOUT publishing

### 2. Publish to crates.io

Once the dry run succeeds:

```bash
cargo publish
```

**Note:** Publishing is **permanent**. You cannot:
- Delete a published version
- Modify a published version
- Re-use a version number

You can only "yank" a version (prevents new projects from using it).

### 3. Verify Publication

1. Check https://crates.io/crates/stm32f411ve-disco
2. Verify documentation at https://docs.rs/stm32f411ve-disco
3. Wait a few minutes for docs.rs to build documentation

## Post-Publishing

### 1. Update README

Add the crates.io badge to README.md:

```markdown
[![Crates.io](https://img.shields.io/crates/v/stm32f411ve-disco.svg)](https://crates.io/crates/stm32f411ve-disco)
[![Documentation](https://docs.rs/stm32f411ve-disco/badge.svg)](https://docs.rs/stm32f411ve-disco)
```

### 2. Announce

Consider announcing in:
- Embedded Rust community forums
- This Week in Rust newsletter
- Twitter/Mastodon with #rustlang #embedded

### 3. Set Up CI/CD

Add GitHub Actions to automate testing:
- Build checks
- Clippy lints
- Documentation builds
- Example compilation

## Versioning Guidelines

Use [Semantic Versioning](https://semver.org/):

- **0.1.0** → Initial release (current)
- **0.1.1** → Backward-compatible bug fixes
- **0.2.0** → New features (backward compatible)
- **1.0.0** → Stable API commitment

For embedded crates, many projects stay at 0.x for a long time.

## Updating Published Crates

When releasing updates:

```bash
# Update version in Cargo.toml
# Update CHANGELOG.md

# Commit changes
git commit -am "Bump version to 0.1.1"

# Tag release
git tag -a v0.1.1 -m "Release v0.1.1"

# Push
git push origin main
git push origin v0.1.1

# Publish
cargo publish
```

## Common Issues

### Issue: "crate name already exists"
Solution: Choose a different name in Cargo.toml

### Issue: "documentation failed to build"
Solution: Run `cargo doc --no-deps` locally to debug

### Issue: "missing license file"
Solution: Ensure LICENSE-MIT and LICENSE-APACHE exist

### Issue: "package size too large"
Solution: Add patterns to `.cargo-ok` or exclude in Cargo.toml:
```toml
[package]
exclude = ["docs/*.pdf", "*.png"]
```

## Helpful Resources

- [Cargo Book - Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io Policy](https://crates.io/policies)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [docs.rs Documentation](https://docs.rs/about)

## Unstable Features Note

This crate uses `forced-target` which is an unstable Cargo feature.
This is fine for publishing - crates.io accepts crates with unstable
features. Users will need to use nightly Cargo to build, which is
standard for embedded Rust projects.
