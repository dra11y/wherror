readme:
    @which cargo-readme || cargo binstall cargo-readme
    cargo readme --output README.md

test:
    cargo test

fix:
    cargo clippy --fix --allow-dirty --allow-staged -- -D warnings
    cargo fmt --all

preflight:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "🔍 Running preflight checks for version bump..."

    # Get current version from Cargo.toml
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    IMPL_VERSION=$(grep '^version = ' impl/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

    echo "📦 Current version in Cargo.toml: $CURRENT_VERSION"
    echo "📦 Current version in impl/Cargo.toml: $IMPL_VERSION"

    # Check that both Cargo.toml versions match
    if [ "$CURRENT_VERSION" != "$IMPL_VERSION" ]; then
        echo "❌ Version mismatch: Cargo.toml ($CURRENT_VERSION) != impl/Cargo.toml ($IMPL_VERSION)"
        exit 1
    fi

    # Get latest version from crates.io
    echo "🌐 Checking latest version on crates.io..."
    LATEST_CRATES_VERSION=$(cargo search wherror --limit 1 2>/dev/null | grep '^wherror = ' | sed 's/wherror = "\(.*\)".*/\1/' || echo "0.0.0")
    echo "📦 Latest version on crates.io: $LATEST_CRATES_VERSION"

    # Check that we've bumped the version
    if [ "$CURRENT_VERSION" = "$LATEST_CRATES_VERSION" ]; then
        echo "❌ Version not bumped: current version ($CURRENT_VERSION) matches crates.io version"
        exit 1
    fi

    # Check for old version references outside CHANGELOG.md
    echo "🔍 Checking for old version references..."
    OLD_VERSION_FILES=$(grep -r "$LATEST_CRATES_VERSION" --exclude-dir=target --exclude-dir=.git --exclude=CHANGELOG.md . | grep -v "Cargo.lock" || true)
    if [ -n "$OLD_VERSION_FILES" ]; then
        echo "❌ Found references to old version ($LATEST_CRATES_VERSION) in:"
        echo "$OLD_VERSION_FILES"
        exit 1
    fi

    # Check CHANGELOG.md has entry for current version
    echo "📝 Checking CHANGELOG.md..."
    if ! grep -q "## \[$CURRENT_VERSION\]" CHANGELOG.md; then
        echo "❌ No changelog entry found for version $CURRENT_VERSION"
        exit 1
    fi

    # Check git status - ensure all changes are committed
    echo "📋 Checking git status..."
    if [ -n "$(git status --porcelain)" ]; then
        echo "❌ Uncommitted changes found. Please commit all changes before release."
        git status
        exit 1
    fi

    # Check that git tag exists for current version
    echo "🏷️  Checking git tag..."
    if ! git tag -l | grep -q "^v$CURRENT_VERSION$"; then
        echo "❌ Git tag v$CURRENT_VERSION not found. Please tag the release:"
        echo "   git tag v$CURRENT_VERSION"
        echo "   git push origin v$CURRENT_VERSION"
        exit 1
    fi

    echo "✅ All preflight checks passed! Ready to publish version $CURRENT_VERSION"

publish *args: test fix readme preflight
    cargo +nightly publish -Z package-workspace --package wherror-impl --package wherror {{args}}
