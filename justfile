# project justfile

import? '.just/shellcheck.just'
import? '.just/compliance.just'
import? '.just/gh-process.just'
import? '.just/cue-verify.just'

# just list (default)
list_recipes:
  just --list

# run the code and see how it goes
[group('Rust')]
try:
  just with_secrets "cargo run"

# run with backtrace enabled
[group('Rust')]
backtrace:
  RUST_BACKTRACE=1 just with_secrets "cargo run"

# what have you broken?
[group('Rust')]
check:
  cargo fmt --check
  cargo check
  cargo clippy
  cargo test --workspace
  cargo audit

# add a crate dependancy
[group('Rust')]
newdep crate_name:
  cargo add {{crate_name}}
  cargo doc

# run cargo with OpenSRS credentials available
[group('Rust')]
run_with_creds subcommand="sync":
  just with_secrets "cargo run {{subcommand}}"

# run tests with OpenSRS credentials available
[group('Rust')]
test_with_creds:
  just with_secrets "cargo test"

# verify 1Password CLI is authenticated
[group('Secrets')]
op_status:
  @op account list || echo "Not signed in. Run: just op_signin"

# sign in to 1Password (interactive)
[group('Secrets')]
op_signin:
  op signin

# test fnox can retrieve OpenSRS credentials
[group('Secrets')]
fnox_test:
  @echo "Testing fnox retrieval..."
  @fnox get OPENSRS_USERNAME || echo "Failed to retrieve username"
  @fnox get OPENSRS_CREDENTIAL || echo "Failed to retrieve credential"

# export secrets to environment and run a command
[group('Secrets')]
with_secrets command:
  #!/usr/bin/env bash
  set -euo pipefail
  export OPENSRS_USERNAME=$(fnox get OPENSRS_USERNAME | tail -1)
  export OPENSRS_CREDENTIAL=$(fnox get OPENSRS_CREDENTIAL | tail -1)
  {{ command }}

# preview what cargo-dist will build (dry run)
[group('Release')]
dist_plan:
  ~/.cargo/bin/dist plan

# build release artifacts locally (for testing)
[group('Release')]
dist_build:
  ~/.cargo/bin/dist build

# verify cargo-dist configuration
[group('Release')]
dist_check:
  ~/.cargo/bin/dist init --yes

# make a release with binaries (cargo-dist)
[group('Release')]
rust_release rel_version:
    #!/usr/bin/env bash
    set -euo pipefail

    # Validate version format (vX.Y.Z)
    if [[ ! "{{rel_version}}" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: Version must be in format vX.Y.Z (e.g., v0.1.0)"
        exit 1
    fi

    echo "Creating and pushing tag {{rel_version}}..."                                                                          git tag "{{rel_version}}"
    git push origin "{{rel_version}}"

    echo "Tag pushed. GitHub Actions will build binaries and create release."
    echo "Watch progress: https://github.com/fini-net/macaw/actions"

    sleep 5
    git pull
