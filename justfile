# Requires:
# - `just` (`cargo install just`)
# - `cargo-edit` (`cargo install cargo-edit`)

# Bump patch version of the crate in Cargo.toml
bump-patch:
    cargo set-version --bump patch

# Increments the minor version of the crate in Cargo.toml
bump-minor:
    cargo set-version --bump minor

# Increments the major version of the crate in Cargo.toml
bump-major:
    cargo set-version --bump major

# Bump minor, create a git tag, and push
release-minor:
    #!/usr/bin/env bash
    set -x
    just bump-minor
    version=$(cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')
    git commit -am "Release v$version" || true
    git tag "v$version"
    git push
    git push origin "v$version"

