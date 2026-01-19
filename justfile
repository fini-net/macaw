# project justfile

import? '.just/shellcheck.just'
import? '.just/compliance.just'
import? '.just/gh-process.just'
import? '.just/cue-verify.just'

# just list (default)
list_recipes:
  just --list

# run the code and see how it goes (default)
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
run_with_creds:
  just with_secrets "cargo run"

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
