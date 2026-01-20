# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Purpose

Macaw is a domain registration backend that integrates with the OpenSRS API. It provides multi-customer domain management with SQLite caching, billing, audit trails, and Authelia authentication integration. The system is designed to be written in Rust using sea-orm for database operations.

## Project Status

Complete:

- Database schema design (SQLite)
- OpenSRS API integration (authentication and domain listing)
- Configuration management with environment variable loading
- MD5-based signature authentication for OpenSRS
- Custom XML serialization/deserialization for OpenSRS API

Remaining:

- Database caching layer (sea-orm integration)
- Multi-customer support
- Authelia authentication integration

## Database Architecture

### Schema Design

The project uses a normalized SQLite database (3NF) with intentional denormalizations for performance. Schema files are located in `docs/`:

- `docs/schema.sql` - Complete SQLite schema with all tables, indexes, and constraints
- `docs/database-schema.md` - Detailed documentation of each table and design decisions
- `docs/erd.md` - Entity relationship diagram in Mermaid format

### Core Database Tables

**Customer & Contact Management:**

- `customers` - Customer accounts linked to Authelia usernames
- `contacts` - Reusable contact information (registrant, admin, tech, billing)
- `domain_contacts` - Junction table linking domains to contact roles

**Domain Management:**

- `domains` - Core domain registration data with status lifecycle
- `nameservers` - DNS nameserver configuration
- `tld_data` - Flexible key-value store for registry-specific requirements (e.g., .ca registrant type, .us nexus category)

**Billing:**

- `invoices` - Customer invoices with denormalized totals
- `billing_items` - Line items for domain services (registration, renewal, transfer, privacy)
- `payments` - Payment transactions with optional invoice linkage

**Audit:**

- `audit_log` - Complete change tracking with JSON snapshots of old/new values

### Key Design Decisions

1. **Intentional Denormalizations** (for query performance):
   - `domains.tld` extracted from `domain_name` for efficient TLD queries
   - `invoices.total_amount` and `paid_amount` cached from sums
   - `customers.account_balance` cached for quick access

2. **SQLite-Specific Implementation**:
   - BOOLEAN stored as INTEGER (0/1)
   - DATETIME stored as TEXT in ISO 8601 format
   - DECIMAL stored as TEXT or REAL (application handles precision)
   - Foreign keys must be enabled: `PRAGMA foreign_keys = ON;`

3. **Contact Reusability**: Contacts can be shared across multiple domains to avoid data duplication while maintaining data integrity

4. **TLD Flexibility**: The `tld_data` table uses key-value pairs to support registry-specific requirements without schema changes

### Database Initialization

```bash
# Create database from schema
sqlite3 macaw.db < docs/schema.sql

# Verify foreign keys are enabled
sqlite3 macaw.db "PRAGMA foreign_keys;"
```

## Rust Code Architecture

### Module Structure

The codebase is organized into two main modules:

- `src/config/` - Configuration management
  - `OpenSrsCredentials::from_env()` - Loads credentials from environment variables
  - `OpenSrsCredentials::available()` - Checks if credentials are present without loading
  - Validates that credentials are non-empty

- `src/opensrs/` - OpenSRS API client implementation
  - `auth.rs` - MD5 signature generation: `md5(md5(xml + key) + key)`
  - `client.rs` - HTTP client with ureq, sends authenticated requests
  - `domain.rs` - High-level domain operations (automatic pagination)
  - `error.rs` - Error types using thiserror (ApiError, HttpError, XmlDeserialize, etc.)
  - `types.rs` - Request/response types (Environment, ClientConfig, ExpiringDomain)
  - `xml.rs` - Custom XML serialization for OpenSRS's non-standard `<dt_assoc>` format

### OpenSRS XML Format

OpenSRS uses a proprietary XML structure with `<dt_assoc>` and `<item key="...">` tags that cannot be handled by standard serde. Manual XML construction and parsing is used:

- Serialization: String building for requests (see `xml::serialize_request`)
- Deserialization: Event-based parsing with quick-xml (see `xml::deserialize_response`)

### API Client Pattern

The `OpenSrsClient` provides high-level methods that handle:

1. XML serialization
2. MD5 signature generation
3. HTTP headers (X-Username, X-Signature)
4. Response parsing
5. Automatic pagination (see `get_domains_by_expiredate`)

### Testing Strategy

Tests use `serial_test` crate for environment variable isolation. Integration tests requiring credentials check `OpenSrsCredentials::available()` and skip if unavailable.

## Technology Stack

- **Language**: Rust 2024 edition
- **HTTP Client**: ureq 3.0 (blocking, with native-tls)
- **XML**: quick-xml 0.36 with manual serialization
- **Hashing**: md-5 0.10 for OpenSRS signature
- **Error Handling**: thiserror 2.0
- **Date/Time**: chrono 0.4
- **Database** (planned): sea-orm with SQLite

### sea-orm Integration (Future)

When implementing the Rust backend:

```bash
# Install sea-orm CLI
cargo install sea-orm-cli

# Generate entities from existing schema
sea-orm-cli generate entity \
    --database-url sqlite://macaw.db \
    --output-dir src/entities

# Run migrations (when created)
sea-orm-cli migrate up
```

Key sea-orm considerations from `docs/database-schema.md`:

- Map SQLite BOOLEAN (INTEGER) to Rust `bool` types
- Use `DeriveActiveEnum` for status/type enums with CHECK constraints
- Implement audit logging in application layer for all INSERT/UPDATE/DELETE operations
- Always use parameterized queries (sea-orm does this automatically)
- Encrypt `domains.auth_code` at rest (transfer authorization codes are sensitive)

## Credential Management

### OpenSRS API Credentials

Macaw retrieves OpenSRS API credentials from 1Password using [fnox](https://github.com/jdx/fnox), a multi-provider secret manager.

**1Password Setup:**

- **Item Name:** "fini-opensrs"
- **Vault:** "Private"
- **Fields Required:**
  - `username` - OpenSRS API username
  - `credential` - OpenSRS API credential (note: not "password"!)

**Developer Workflow:**

```bash
# One-time per terminal session
just op_signin

# Verify credentials are accessible
just fnox_test

# Run application with credentials
just run_with_creds

# Run tests with credentials
just test_with_creds
```

**How It Works:**

1. `fnox.toml` defines secret references (safe to commit)
2. `just` recipes use `fnox get` to retrieve secrets into environment
3. Rust code reads from `OPENSRS_USERNAME` and `OPENSRS_CREDENTIAL` env vars
4. Configuration module validates and provides type-safe access

**CI/CD Considerations:**

For automated testing without 1Password access:

- Set environment variables directly in CI
- Use mock credentials for unit tests
- Integration tests can be skipped if credentials unavailable

**Security Notes:**

- Credentials never stored in git (only references)
- User session authentication (no service account tokens)
- fnox.toml is safe to commit
- Credentials only live in environment variables at runtime

## Development Workflow

This repository uses `just` (command runner) for all development tasks. The workflow is entirely command-line based using `just` and the GitHub CLI (`gh`).

### Standard Development Cycle

1. `just branch <name>` - Create a new feature branch (format: `$USER/YYYY-MM-DD-<name>`)
2. Make changes and commit (last commit message becomes PR title)
3. `just pr` - Create PR, push changes, and watch checks (waits 8s for GitHub API)
4. `just merge` - Squash merge PR, delete branch, return to main, and pull latest
5. `just sync` - Return to main branch and pull latest (escape hatch)

### Additional Commands

- `just` or `just list` - Show all available recipes
- `just prweb` - Open current PR in browser
- `just release <version>` - Create a GitHub release with auto-generated notes
- `just compliance_check` - Run custom repo compliance checks
- `just cue-verify` - Verify .repo.toml structure and flag configuration
- `just shellcheck` - Run shellcheck on all bash scripts in just recipes
- `just utcdate` - Print UTC date in ISO format (used in branch names)

### Modular Justfile Structure

The main `justfile` imports five modules from `.just/`:

- `.just/compliance.just` - Custom compliance checks for repo health
- `.just/gh-process.just` - Git/GitHub workflow automation (core PR lifecycle)
- `.just/pr-hook.just` - Optional pre-PR hooks for project-specific automation
- `.just/shellcheck.just` - Shellcheck linting for bash scripts in just recipes
- `.just/cue-verify.just` - CUE validation for .repo.toml structure and flags

### Repository Configuration

The `.repo.toml` file defines repository metadata and feature flags:

```toml
[about]
description = "domain registration backend"
license = "GPL-2.0-only"

[urls]
git_ssh = "git@github.com:fini-net/macaw.git"
web_url = "https://github.com/fini-net/macaw"

[flags]
claude = true
claude-review = true
copilot-review = true
```

Validation is enforced via CUE schema in `docs/repo-toml.cue` and verified with `just cue-verify`.

## Development Commands

### Quality Checks

```bash
just check         # Run all checks: fmt, clippy, test, audit
cargo fmt --check  # Check code formatting
cargo check        # Compile check without building
cargo clippy       # Run linter
cargo test         # Run all tests
cargo audit        # Check for security vulnerabilities
```

### Running the Application

```bash
just try              # Run with credentials (default)
just run_with_creds   # Run with OpenSRS credentials from 1Password
just backtrace        # Run with RUST_BACKTRACE=1 for detailed errors
```

### Testing

```bash
just test_with_creds  # Run tests with OpenSRS credentials
cargo test            # Run tests without credentials (some will skip)
```

### Environment Variables

- `OPENSRS_USERNAME` - OpenSRS API username (loaded by fnox)
- `OPENSRS_CREDENTIAL` - OpenSRS API credential (loaded by fnox)
- `OPENSRS_ENVIRONMENT` - Set to "production" for production API (defaults to test)

## OpenSRS Integration

### Current Implementation

The OpenSRS client (`src/opensrs/`) handles:

- Authentication via MD5 signature (`X-Username` and `X-Signature` headers)
- Environment selection (test/production endpoints)
- Domain listing with automatic pagination
- Custom XML format serialization/deserialization

### Future Database Caching

The system will cache OpenSRS domain data locally to:

- Reduce API calls and improve response times
- Support offline reporting and analytics
- Maintain audit trails of all domain changes
- Enable efficient querying by customer, TLD, expiration date, etc.

Registry-specific data (like .ca or .us requirements) is stored in the flexible `tld_data` table.

## Security Considerations

When implementing the backend:

1. **SQL Injection**: sea-orm uses parameterized queries automatically
2. **Sensitive Data**:
   - `domains.auth_code` (EPP transfer codes) should be encrypted at rest
   - Never store payment card details - only store `transaction_id` references
   - Passwords handled by Authelia (external authentication)
3. **Row-Level Security**: Always filter queries by `customer_id` to prevent unauthorized access
4. **Audit Logging**: Populate `audit_log` for all domain/contact/billing changes with IP address and user agent

## GitHub Actions Workflows

Seven workflows run on PRs and pushes to main:

- `markdownlint` - Enforces markdown standards using `markdownlint-cli2`
- `checkov` - Security scanning for GitHub Actions (continues on error, outputs SARIF)
- `actionlint` - Lints GitHub Actions workflow files
- `cue-verify` - Validates .repo.toml structure and flag configuration
- `auto-assign` - Automatically assigns issues/PRs to `chicks-net`
- `claude-code-review` - Claude AI review automation
- `claude` - Additional Claude integration

Run markdown linting locally: `markdownlint-cli2 **/*.md`

## Markdown Linting

Configuration in `.markdownlint.yml`:

- MD013 (line length) is disabled
- MD041 (first line h1) is disabled
- MD042 (no empty links) is disabled
- MD004 (list style) enforces dashes
- MD010 (tabs) ignores code blocks
