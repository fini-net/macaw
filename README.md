# macaw

![GitHub Issues](https://img.shields.io/github/issues/fini-net/macaw)
![GitHub Pull Requests](https://img.shields.io/github/issues-pr/fini-net/macaw)
![GitHub License](https://img.shields.io/github/license/fini-net/macaw)
![GitHub watchers](https://img.shields.io/github/watchers/fini-net/macaw)

domain registration backend

![macaws take over the data center](docs/macaws-take-over-data-center.jpg)

## Initial Requirements

- [x] works with OpenSRS API
- [ ] cache in a database
- [ ] handle multiple customers
- [ ] compatible with Authelia
- [x] write server in Rust with sea-orm
- [ ] maintain audit journal of all domain changes
- [ ] handle billing
- [ ] contact information updates
- [x] clean interface, no ads

Eventually I'd like to implement most of
[these features](https://github.com/fini-net/macaw/pull/3#issuecomment-3753022343).

## Installation

### Using Shell Installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/fini-net/macaw/releases/latest/download/macaw-installer.sh | sh
```

### Download Pre-built Binaries

From [releases page](https://github.com/fini-net/macaw/releases):

- macOS (Apple Silicon): `macaw-*-aarch64-apple-darwin.tar.gz`
- macOS (Intel): `macaw-*-x86_64-apple-darwin.tar.gz`
- Linux (GNU): `macaw-*-x86_64-unknown-linux-gnu.tar.gz`
- Linux (musl): `macaw-*-x86_64-unknown-linux-musl.tar.gz`

```bash
tar xzf macaw-*.tar.gz
mv macaw ~/.local/bin/
```

## Quick Start

### Prerequisites

- Rust 2024 edition
- [1Password CLI](https://developer.1password.com/docs/cli)
- [fnox](https://github.com/jdx/fnox): `cargo install fnox`
- [just](https://github.com/casey/just) command runner
- Access to "fini-opensrs" item in 1Password

### Setup

1. Clone and enter directory
2. Sign in to 1Password: `just op_signin`
3. Verify credentials: `just fnox_test`
4. Run application: `just run_with_creds`

### Commands

- `just list` - Show all commands
- `just check` - Run quality checks (fmt, clippy, test, audit)
- `just run_with_creds` - Run with OpenSRS credentials
- `just test_with_creds` - Run tests with credentials

## Contributing

- [Code of Conduct](.github/CODE_OF_CONDUCT.md)
- [Contributing Guide](.github/CONTRIBUTING.md) includes a step-by-step guide to our
  [development process](.github/CONTRIBUTING.md#development-process).

## Support & Security

- [Getting Support](.github/SUPPORT.md)
- [Security Policy](.github/SECURITY.md)
