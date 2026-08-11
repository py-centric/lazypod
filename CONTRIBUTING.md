# Contributing

Thank you for your interest in contributing to Lazypod!

## Contributor License Agreement

Before your first pull request can be merged, you must sign the Contributor License Agreement (CLA). The CLA ensures that the project maintainers have the necessary rights to use, modify, and distribute your contributions under the project's license.

**To sign the CLA:**

1. Open a pull request with your changes.
2. A GitHub bot will check whether you have signed the CLA.
3. If you have not signed, the bot will post a comment with a link to sign.
4. Follow the link, fill out the form, and submit.
5. The bot will update the PR status once the CLA is signed.

You only need to sign the CLA once. All future contributions will be covered.

## Local Development Requirements

To contribute to Lazypod, you'll need the following installed:
- **Rust / Cargo** (version 1.75 or newer).
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

## Code Style

All code must pass the project's automated quality checks before it can be merged. These checks run in CI on every pull request.

### Formatting

Lazypod uses `rustfmt` for consistent code formatting. Run it before committing:

```bash
cargo fmt --all
```

To check formatting without modifying files:

```bash
cargo fmt --all -- --check
```

### Linting

Lazypod uses `clippy` for static analysis. All clippy warnings are treated as errors in CI:

```bash
cargo clippy -- -D warnings
```

Fix any warnings before submitting your PR.

### Security

CI runs `cargo audit` to check for known vulnerabilities in dependencies. If a dependency has a known vulnerability, update it or document why the vulnerability does not apply.

## Branching Strategy and Versioning

Lazypod uses a strict branching pattern to determine version bumps according to [SemVer](https://semver.org/). Automated logic in `.github/workflows/bump-version.yml` detects the branch prefix and bumps `Cargo.toml`, `Cargo.lock`, and `docs/conf.py` accordingly.

When creating a new branch, use the following prefixes:

- **Patch Version Bumps (vX.Y.+1):** `fix/*` or `bugfix/*` — backwards-compatible bug fixes.
- **Minor Version Bumps (vX.+1.0):** `feat/*` or `feature/*` — new functionality in a backwards-compatible manner.
- **Major Version Bumps (v+1.0.0):** `release/*` or `breaking/*` — breaking API/usage changes.

## Pull Request Requirements

Before submitting a pull request, ensure the following:

1. **Tests pass**: Run `cargo test` and confirm all tests pass.
2. **Formatting**: Run `cargo fmt --all` and confirm no changes are needed.
3. **Linting**: Run `cargo clippy -- -D warnings` and fix all warnings.
4. **CLA signed**: Ensure you have signed the Contributor License Agreement (the bot will check this automatically).
5. **Branch naming**: Use the correct branch prefix (`fix/`, `feat/`, `release/`, etc.) so version bumping works correctly.
6. **Commit messages**: Write clear, concise commit messages that describe what changed and why.
7. **Documentation**: If your change affects user-facing behavior, update the relevant documentation in `docs/` or `README.md`.

## Semantic Versioning and Automated Bumping

Upon pushing to a prefixed branch, the version bumping GitHub Action will be executed automatically. It reads the current version from `Cargo.toml`, applies the appropriate bump based on the branch prefix, updates `Cargo.toml`, `Cargo.lock`, and `docs/conf.py`, and commits the changes back to the branch.
