# Contributing

Thank you for your interest in contributing to Lazypod!

## Local Development Requirements

To contribute to Lazypod, you'll need the following installed:
- **Rust / Cargo** (version 1.70 or newer).
- **Podman** (Docker is optional but recommended if you want to test the Docker-specific views).
- **Python and Sphinx** (Optional, only if you plan on building the documentation).

### Building and Running the App

Lazypod is built using Cargo. Use the following commands for development:

```bash
# Build the application
cargo build

# Run the application
cargo run

# Build a release binary
cargo build --release
```

### Testing

Make sure you run tests before submitting any code:

```bash
# Run tests
cargo test
```

### Documentation

The project uses Sphinx to build documentation from the `docs/` folder.

```bash
# Navigate to docs
cd docs

# Generate HTML documentation
make html
```

The localized documentation will be available in `docs/_build/html/index.html`.

## Semantic Versioning and Branching Strategy

Lazypod uses a strict branching pattern to determine version bumps according to [SemVer](https://semver.org/). We have automated logic in `.github/workflows/bump-version.yml` which detects the branch prefix you push and bumps `Cargo.toml`, `Cargo.lock`, and `docs/conf.py` accordingly.

When creating a new branch, please adhere to the following prefixes so the Semantic Version logic knows how to bump the release version automatically:

- **Patch Version Bumps (vX.Y.+1):** Use `fix/*` or `bugfix/*` branch names. These branches should be used to fix individual bugs in a backwards compatible way.
- **Minor Version Bumps (vX.+1.0):** Use `feat/*` or `feature/*` branch names. These are used to introduce new functionality in a backwards-compatible manner.
- **Major Version Bumps (v+1.0.0):** Use `release/*` or `breaking/*` branch names. These are meant for big releases containing breaking API/usage changes.

Any standard pull requests you make should follow this branch name structure. Upon pushing to one of these branches, the version bumping Action will be executed automatically!
