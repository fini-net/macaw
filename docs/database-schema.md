# Macaw Database Schema Design

This document describes the normalized SQLite database schema for the Macaw domain registration backend.

## Overview

The schema supports:

- Multi-customer domain registration management
- Integration with Authelia for authentication
- OpenSRS API integration for domain operations
- Contact management with reusable contact records
- Billing and invoicing
- Complete audit trail
- Registry-specific TLD data storage

## Database Tables

### 1. customers

Stores customer account information with Authelia integration.

```sql
CREATE TABLE customers (
    customer_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,  -- Authelia username
    email TEXT NOT NULL,
    company_name TEXT,
    account_balance DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    credit_limit DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    status TEXT NOT NULL CHECK(status IN ('active', 'suspended', 'closed')) DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Indexes:**

- `idx_customers_username` on `username`
- `idx_customers_email` on `email`
- `idx_customers_status` on `status`

**Key Fields:**

- `username`: Links to Authelia authentication system
- `account_balance`: Cached customer account balance
- `credit_limit`: Maximum credit allowed
- `status`: Account status (active, suspended, closed)

### 2. contacts

Stores contact information that can be reused across multiple domains.

```sql
CREATE TABLE contacts (
    contact_id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    contact_type TEXT NOT NULL CHECK(contact_type IN ('registrant', 'admin', 'tech', 'billing')),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    organization TEXT,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    fax TEXT,
    address1 TEXT NOT NULL,
    address2 TEXT,
    city TEXT NOT NULL,
    state_province TEXT NOT NULL,
    postal_code TEXT NOT NULL,
    country_code TEXT NOT NULL,  -- ISO 3166-1 alpha-2
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id) ON DELETE CASCADE
);
```

**Indexes:**

- `idx_contacts_customer` on `customer_id`
- `idx_contacts_email` on `email`
- `idx_contacts_type` on `contact_type`

**Key Fields:**

- `contact_type`: Categorizes contact (registrant, admin, tech, billing)
- `country_code`: ISO 3166-1 alpha-2 country code
- Supports both individual and organization contacts

### 3. domains

Core table storing domain registration data.

```sql
CREATE TABLE domains (
    domain_id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    domain_name TEXT NOT NULL UNIQUE,
    tld TEXT NOT NULL,  -- extracted from domain_name for querying
    status TEXT NOT NULL CHECK(status IN (
        'pending', 'active', 'expired', 'grace', 'redemption',
        'pending_delete', 'transferred_away', 'cancelled'
    )) DEFAULT 'pending',
    registration_date DATETIME NOT NULL,
    expiration_date DATETIME NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT 1,
    transfer_lock BOOLEAN NOT NULL DEFAULT 1,
    auth_code TEXT,
    whois_privacy BOOLEAN NOT NULL DEFAULT 0,
    registry_domain_id TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id) ON DELETE RESTRICT
);
```

**Indexes:**

- `idx_domains_customer` on `customer_id`
- `idx_domains_name` on `domain_name` (unique)
- `idx_domains_tld` on `tld`
- `idx_domains_status` on `status`
- `idx_domains_expiration` on `expiration_date`

**Key Fields:**

- `domain_name`: Fully qualified domain name (FQDN)
- `tld`: Top-level domain extracted for queries
- `status`: Domain lifecycle state
- `auth_code`: EPP authorization code for transfers
- `transfer_lock`: Prevents unauthorized transfers
- `whois_privacy`: Privacy protection flag
- `registry_domain_id`: OpenSRS/registry identifier

**Domain Status Values:**

- `pending`: Registration in progress
- `active`: Domain is active and in good standing
- `expired`: Domain has passed expiration date
- `grace`: In grace period after expiration
- `redemption`: In redemption period
- `pending_delete`: Queued for deletion
- `transferred_away`: Domain transferred to another registrar
- `cancelled`: Registration cancelled

### 4. domain_contacts

Junction table linking domains to contacts with specific roles.

```sql
CREATE TABLE domain_contacts (
    domain_id INTEGER NOT NULL,
    contact_id INTEGER NOT NULL,
    contact_role TEXT NOT NULL CHECK(contact_role IN ('registrant', 'admin', 'tech', 'billing')),
    PRIMARY KEY (domain_id, contact_role),
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(contact_id) ON DELETE RESTRICT
);
```

**Indexes:**

- `idx_domain_contacts_domain` on `domain_id`
- `idx_domain_contacts_contact` on `contact_id`

**Key Fields:**

- Composite primary key ensures each role is filled exactly once per domain
- `contact_role`: registrant, admin, tech, or billing
- Same contact can fill multiple roles for one domain
- Contact can be reused across multiple domains

### 5. nameservers

DNS nameserver configuration for domains.

```sql
CREATE TABLE nameservers (
    nameserver_id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    hostname TEXT NOT NULL,
    ip_address TEXT,  -- Optional glue record
    priority INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE
);
```

**Indexes:**

- `idx_nameservers_domain` on `domain_id`

**Key Fields:**

- `hostname`: Nameserver FQDN
- `ip_address`: Optional glue record (required for in-bailiwick nameservers)
- `priority`: Order/priority of nameservers

### 6. tld_data

Registry-specific data storage using key-value pairs.

```sql
CREATE TABLE tld_data (
    tld_data_id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    data_key TEXT NOT NULL,
    data_value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE
);
```

**Indexes:**

- `idx_tld_data_domain` on `domain_id`
- `idx_tld_data_key` on `data_key`

**Purpose:**

- Flexible storage for TLD-specific requirements (e.g., .ca registrant type, .us nexus category)
- Avoids adding TLD-specific columns to domains table
- Supports future TLDs without schema changes

**Example Data:**

```sql
-- For .ca domain
INSERT INTO tld_data (domain_id, data_key, data_value) VALUES
    (1, 'registrant_type', 'CCT'),  -- Canadian citizen
    (1, 'cira_agreement', '2.0');

-- For .us domain
INSERT INTO tld_data (domain_id, data_key, data_value) VALUES
    (2, 'nexus_category', 'C11'),  -- US citizen
    (2, 'application_purpose', 'P1');  -- Business use
```

### 7. invoices

Customer invoices for domain services.

```sql
CREATE TABLE invoices (
    invoice_id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    invoice_number TEXT NOT NULL UNIQUE,
    invoice_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    due_date DATETIME NOT NULL,
    total_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    paid_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    status TEXT NOT NULL CHECK(status IN ('draft', 'issued', 'paid', 'overdue', 'cancelled')) DEFAULT 'draft',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id) ON DELETE RESTRICT
);
```

**Indexes:**

- `idx_invoices_customer` on `customer_id`
- `idx_invoices_number` on `invoice_number` (unique)
- `idx_invoices_status` on `status`
- `idx_invoices_due_date` on `due_date`

**Key Fields:**

- `invoice_number`: Human-readable unique identifier
- `total_amount`: Cached sum of billing_items (denormalized)
- `paid_amount`: Cached sum of payments (denormalized)
- `status`: Invoice state (draft, issued, paid, overdue, cancelled)

### 8. billing_items

Line items for invoices.

```sql
CREATE TABLE billing_items (
    billing_item_id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL,
    domain_id INTEGER,  -- Optional reference to domain
    item_type TEXT NOT NULL CHECK(item_type IN ('registration', 'renewal', 'transfer', 'privacy', 'other')),
    description TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price DECIMAL(10,2) NOT NULL,
    total_price DECIMAL(10,2) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (invoice_id) REFERENCES invoices(invoice_id) ON DELETE CASCADE,
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE SET NULL
);
```

**Indexes:**

- `idx_billing_items_invoice` on `invoice_id`
- `idx_billing_items_domain` on `domain_id`

**Key Fields:**

- `item_type`: Category of charge
- `domain_id`: Optional link to specific domain
- `total_price`: quantity × unit_price

### 9. payments

Payment transactions from customers.

```sql
CREATE TABLE payments (
    payment_id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    invoice_id INTEGER,  -- Optional invoice link
    payment_method TEXT NOT NULL CHECK(payment_method IN ('credit_card', 'bank_transfer', 'paypal', 'credit', 'other')),
    transaction_id TEXT,  -- External payment processor ID
    amount DECIMAL(10,2) NOT NULL,
    payment_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL CHECK(status IN ('pending', 'completed', 'failed', 'refunded')) DEFAULT 'pending',
    notes TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (customer_id) REFERENCES customers(customer_id) ON DELETE RESTRICT,
    FOREIGN KEY (invoice_id) REFERENCES invoices(invoice_id) ON DELETE SET NULL
);
```

**Indexes:**

- `idx_payments_customer` on `customer_id`
- `idx_payments_invoice` on `invoice_id`
- `idx_payments_transaction` on `transaction_id`
- `idx_payments_status` on `status`

**Key Fields:**

- `payment_method`: How payment was made
- `transaction_id`: External payment processor reference
- `invoice_id`: Optional link (allows account credits)
- `status`: Payment state

### 10. audit_log

Complete audit trail for all database changes.

```sql
CREATE TABLE audit_log (
    audit_id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    record_id INTEGER NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('INSERT', 'UPDATE', 'DELETE')),
    changed_by TEXT NOT NULL,  -- Authelia username
    old_values TEXT,  -- JSON
    new_values TEXT,  -- JSON
    ip_address TEXT,
    user_agent TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Indexes:**

- `idx_audit_table_record` on `(table_name, record_id)`
- `idx_audit_changed_by` on `changed_by`
- `idx_audit_timestamp` on `timestamp`

**Key Fields:**

- `table_name` + `record_id`: Identifies affected record
- `action`: Type of change (INSERT, UPDATE, DELETE)
- `changed_by`: Authelia username who made the change
- `old_values`/`new_values`: JSON snapshots of data before/after change
- `ip_address`/`user_agent`: Request metadata

## Normalization Analysis

### First Normal Form (1NF)

✅ **Satisfied**: All tables have:

- Atomic columns (no repeating groups or arrays)
- Primary keys defined
- No duplicate rows possible

### Second Normal Form (2NF)

✅ **Satisfied**: All non-key attributes depend on the entire primary key.

Example: In `domain_contacts`, the composite key is `(domain_id, contact_role)`. There are no non-key attributes that depend on only part of the key.

### Third Normal Form (3NF)

✅ **Satisfied**: No transitive dependencies exist. All non-key attributes depend only on the primary key.

### Intentional Denormalizations

For performance optimization, three denormalizations are present:

1. **domains.tld** - Extracted from `domain_name`
   - Rationale: Enables efficient queries by TLD without parsing domain_name
   - Maintained by: Application logic or trigger on INSERT/UPDATE

2. **invoices.total_amount** and **invoices.paid_amount** - Cached sums
   - Rationale: Avoids expensive SUM() queries on billing_items and payments
   - Maintained by: Application logic or triggers

3. **customers.account_balance** - Cached balance
   - Rationale: Frequent access pattern, expensive to calculate from invoices/payments
   - Maintained by: Application logic or triggers

## SQLite Specific Implementation Notes

### Data Types

SQLite has dynamic typing, but the schema uses type affinities:

- `INTEGER`: For IDs and quantities
- `TEXT`: For strings (no length limit in SQLite)
- `DECIMAL(10,2)`: Stored as TEXT or REAL (application handles precision)
- `BOOLEAN`: Stored as INTEGER (0 = false, 1 = true)
- `DATETIME`: Stored as TEXT in ISO 8601 format

### Boolean Handling

SQLite doesn't have a native BOOLEAN type. The schema uses:

```sql
BOOLEAN NOT NULL DEFAULT 1  -- Stored as INTEGER (0 or 1)
```

In sea-orm, map these to Rust `bool` types.

### Foreign Keys

Enable foreign key support in SQLite:

```sql
PRAGMA foreign_keys = ON;
```

This must be executed for each database connection.

### Auto-increment

SQLite's `AUTOINCREMENT` ensures primary keys are never reused, even after deletion.

## sea-orm Integration

### Entity Generation

Generate Rust entities from the schema:

```bash
# Install sea-orm-cli
cargo install sea-orm-cli

# Generate entities
sea-orm-cli generate entity \
    --database-url sqlite://macaw.db \
    --output-dir src/entities
```

### Example Entity Code

```rust
// entities/customers.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "customers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub customer_id: i32,
    #[sea_orm(unique)]
    pub username: String,
    pub email: String,
    pub company_name: Option<String>,
    pub account_balance: Decimal,
    pub credit_limit: Decimal,
    pub status: CustomerStatus,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum CustomerStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "suspended")]
    Suspended,
    #[sea_orm(string_value = "closed")]
    Closed,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::contacts::Entity")]
    Contacts,
    #[sea_orm(has_many = "super::domains::Entity")]
    Domains,
    #[sea_orm(has_many = "super::invoices::Entity")]
    Invoices,
    #[sea_orm(has_many = "super::payments::Entity")]
    Payments,
}

impl Related<super::contacts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contacts.def()
    }
}

// Similar implementations for other relations...
```

### Migration Setup

Create migrations in `migration/src/`:

```rust
// migration/src/m20240101_000001_create_customers_table.rs
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000001_create_customers_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Customers::Table)
                    .col(
                        ColumnDef::new(Customers::CustomerId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Customers::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    // ... more columns
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Customers::Table).to_owned())
            .await
    }
}
```

### Query Examples

```rust
use sea_orm::*;
use entities::{customers, domains, prelude::*};

// Find customer by username
let customer: Option<customers::Model> = Customers::find()
    .filter(customers::Column::Username.eq("alice"))
    .one(db)
    .await?;

// Find all active domains for a customer
let domains: Vec<domains::Model> = Domains::find()
    .filter(domains::Column::CustomerId.eq(customer_id))
    .filter(domains::Column::Status.eq("active"))
    .order_by_asc(domains::Column::ExpirationDate)
    .all(db)
    .await?;

// Find domains expiring soon (with eager loading of customer)
let expiring: Vec<(domains::Model, Option<customers::Model>)> = Domains::find()
    .find_also_related(Customers)
    .filter(domains::Column::ExpirationDate.lt(Expr::current_timestamp().add(Interval::days(30))))
    .filter(domains::Column::Status.eq("active"))
    .all(db)
    .await?;

// Create new domain with transaction
let txn = db.begin().await?;

let domain = domains::ActiveModel {
    customer_id: Set(customer_id),
    domain_name: Set("example.com".to_owned()),
    tld: Set("com".to_owned()),
    status: Set("pending".to_owned()),
    registration_date: Set(Utc::now()),
    expiration_date: Set(Utc::now() + Duration::days(365)),
    ..Default::default()
};

let result = domain.insert(&txn).await?;

// Create audit log entry
let audit = audit_log::ActiveModel {
    table_name: Set("domains".to_owned()),
    record_id: Set(result.domain_id),
    action: Set("INSERT".to_owned()),
    changed_by: Set(username.clone()),
    new_values: Set(serde_json::to_string(&result)?),
    ..Default::default()
};

audit.insert(&txn).await?;

txn.commit().await?;
```

## Performance Optimization

### Indexing Strategy

All foreign keys are indexed, plus additional indexes on:

- Frequently queried fields (status, username, email)
- Fields used in ORDER BY clauses (expiration_date, due_date)
- Unique constraints (domain_name, invoice_number)

### Query Optimization Tips

1. **Use indexes for WHERE clauses**: All indexed columns support efficient lookups
2. **Limit result sets**: Use `.limit()` and `.offset()` for pagination
3. **Eager loading**: Use `.find_also_related()` to avoid N+1 queries
4. **Partial selects**: Use `.select_only()` when not all columns needed
5. **Batch operations**: Use transactions for multiple related inserts/updates

### Caching Considerations

Consider caching:

- Customer account balances (already denormalized)
- Active domain counts per customer
- Invoice totals (already denormalized)
- Frequently accessed contact information

## Security Considerations

### SQL Injection Prevention

sea-orm uses parameterized queries, preventing SQL injection:

```rust
// SAFE - parameterized
let domain = Domains::find()
    .filter(domains::Column::DomainName.eq(user_input))
    .one(db)
    .await?;

// UNSAFE - never do this
let query = format!("SELECT * FROM domains WHERE domain_name = '{}'", user_input);
```

### Sensitive Data

- **auth_code**: Domain transfer authorization codes should be encrypted at rest
- **payment information**: Store only references (transaction_id), not card details
- **passwords**: Never store passwords (use Authelia for authentication)

### Access Control

Implement row-level security in application:

```rust
// Ensure user can only access their own domains
let domain = Domains::find()
    .filter(domains::Column::DomainId.eq(domain_id))
    .filter(domains::Column::CustomerId.eq(current_customer_id))  // Security check
    .one(db)
    .await?;
```

### Audit Logging

The `audit_log` table captures all changes. Implement triggers or application logic to populate it:

```rust
// After any domain update
let audit = audit_log::ActiveModel {
    table_name: Set("domains".to_owned()),
    record_id: Set(domain_id),
    action: Set("UPDATE".to_owned()),
    changed_by: Set(session.username.clone()),
    old_values: Set(serde_json::to_string(&old_domain)?),
    new_values: Set(serde_json::to_string(&new_domain)?),
    ip_address: Set(Some(request.ip().to_string())),
    user_agent: Set(request.headers().get("user-agent").map(|h| h.to_str().unwrap().to_owned())),
    ..Default::default()
};
audit.insert(db).await?;
```

## Migration Strategy

### Initial Setup

1. Create SQLite database:

   ```bash
   sqlite3 macaw.db < docs/schema.sql
   ```

2. Enable foreign keys in application:

   ```rust
   let db = Database::connect("sqlite://macaw.db?mode=rwc").await?;
   db.execute_unprepared("PRAGMA foreign_keys = ON;").await?;
   ```

3. Generate entities:

   ```bash
   sea-orm-cli generate entity --database-url sqlite://macaw.db --output-dir src/entities
   ```

### Schema Changes

For schema changes, create sea-orm migrations:

```rust
// migration/src/m20240201_000001_add_domain_privacy_flag.rs
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Domains::Table)
                    .add_column(
                        ColumnDef::new(Domains::WhoisPrivacy)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }
}
```

Run migrations:

```bash
sea-orm-cli migrate up
```

## Testing Strategy

### Test Data

Use the sample data in `schema.sql` for testing:

```sql
-- Uncomment sample data section in schema.sql
-- Or create test fixtures
```

### Integration Tests

```rust
#[cfg(test)]
mod tests {
    use sea_orm::*;

    #[tokio::test]
    async fn test_create_customer() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        // Run migrations
        // Insert test customer
        // Assert customer was created
    }

    #[tokio::test]
    async fn test_domain_expiration_query() {
        // Setup test database
        // Insert domains with various expiration dates
        // Query for expiring domains
        // Assert correct domains returned
    }
}
```

## Future Enhancements

Potential schema extensions:

1. **DNS Zone Management**: Add tables for DNS records (A, AAAA, MX, TXT, etc.)
2. **Email Forwarding**: Table for email aliases and forwarding rules
3. **SSL Certificates**: Track SSL cert purchases and renewals
4. **Reseller Support**: Add reseller hierarchy and commission tracking
5. **Domain Parking**: Parking page templates and statistics
6. **Bulk Operations**: Table for tracking bulk domain operations
7. **API Keys**: Store customer API credentials for programmatic access
8. **Notifications**: Track email/SMS notifications sent to customers
9. **Domain Backordering**: Queue for attempting to register expiring domains
10. **DNSSEC**: Store DS records and DNSSEC status

## References

- [OpenSRS API Documentation](https://domains.opensrs.guide/)
- [sea-orm Documentation](https://www.sea-ql.org/SeaORM/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [ISO 3166-1 Country Codes](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)
- [EPP (Extensible Provisioning Protocol)](https://tools.ietf.org/html/rfc5730)
