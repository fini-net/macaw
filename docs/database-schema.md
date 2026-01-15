# Database Schema Design for Macaw

## Overview

This document describes the normalized SQLite database schema for the Macaw domain registration backend system. The schema is designed to support OpenSRS API integration, multi-customer management, billing, audit logging, and contact management.

## Design Principles

- **Third Normal Form (3NF)** - All tables are normalized to eliminate redundancy
- **SQLite Compatible** - Uses SQLite-compatible data types and features
- **Audit Trail** - Complete change history with `audit_log` table
- **Multi-tenancy** - Support for multiple customers via `customers` table
- **Referential Integrity** - Foreign key constraints maintain data consistency

## Entity Relationship Diagram (ASCII)

```
┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│  customers   │         │   domains    │         │   contacts   │
├──────────────┤         ├──────────────┤         ├──────────────┤
│ id (PK)      │◄────┐   │ id (PK)      │    ┌───►│ id (PK)      │
│ email        │     │   │ customer_id  │────┘    │ type         │
│ authelia_id  │     └───│ (FK)         │         │ name         │
│ created_at   │         │ domain_name  │         │ organization │
│ ...          │         │ status       │         │ email        │
└──────────────┘         │ ...          │         │ phone        │
                         └──────┬───────┘         │ ...          │
                                │                 └──────────────┘
                                │
                    ┌───────────┴───────────┐
                    │                       │
            ┌───────▼──────┐       ┌───────▼──────────┐
            │ nameservers  │       │ domain_contacts  │
            ├──────────────┤       ├──────────────────┤
            │ id (PK)      │       │ domain_id (FK)   │
            │ domain_id(FK)│       │ contact_id (FK)  │
            │ hostname     │       │ contact_role     │
            │ ip_address   │       └──────────────────┘
            │ ...          │
            └──────────────┘

┌──────────────┐         ┌──────────────┐         ┌──────────────┐
│   invoices   │         │billing_items │         │  payments    │
├──────────────┤         ├──────────────┤         ├──────────────┤
│ id (PK)      │         │ id (PK)      │         │ id (PK)      │
│ customer_id  │◄────┐   │ invoice_id   │────┐    │ invoice_id   │
│ (FK)         │     └───│ (FK)         │    └───►│ (FK)         │
│ invoice_date │         │ domain_id(FK)│         │ amount       │
│ due_date     │         │ description  │         │ paid_at      │
│ total_amount │         │ amount       │         │ ...          │
│ status       │         │ ...          │         └──────────────┘
└──────────────┘         └──────────────┘

┌──────────────┐         ┌──────────────┐
│  audit_log   │         │  tld_data    │
├──────────────┤         ├──────────────┤
│ id (PK)      │         │ id (PK)      │
│ table_name   │         │ domain_id(FK)│
│ record_id    │         │ tld          │
│ operation    │         │ key          │
│ old_values   │         │ value        │
│ new_values   │         │ ...          │
│ changed_by   │         └──────────────┘
│ changed_at   │
└──────────────┘
```

## Table Definitions

### customers

Stores customer/reseller account information.

```sql
CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE,
    authelia_id TEXT UNIQUE,
    company_name TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'suspended', 'closed')),
    account_balance REAL NOT NULL DEFAULT 0.0,
    credit_limit REAL NOT NULL DEFAULT 0.0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_customers_email ON customers(email);
CREATE INDEX idx_customers_authelia_id ON customers(authelia_id);
CREATE INDEX idx_customers_status ON customers(status);
```

**Normalization Notes:**
- Email and authelia_id are unique to prevent duplicate accounts
- Status uses CHECK constraint for data integrity
- Timestamps use ISO 8601 format via SQLite datetime()

---

### contacts

Stores contact information for domain registrations (registrant, admin, technical, billing).

```sql
CREATE TABLE contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    type TEXT NOT NULL CHECK(type IN ('registrant', 'admin', 'technical', 'billing')),
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    organization TEXT,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    fax TEXT,
    address1 TEXT NOT NULL,
    address2 TEXT,
    city TEXT NOT NULL,
    state_province TEXT,
    postal_code TEXT NOT NULL,
    country_code TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE
);

CREATE INDEX idx_contacts_customer_id ON contacts(customer_id);
CREATE INDEX idx_contacts_type ON contacts(type);
CREATE INDEX idx_contacts_email ON contacts(email);
```

**Normalization Notes:**
- Contacts are separate entities that can be reused across multiple domains
- Type field allows same contact entity to serve multiple roles
- Foreign key cascade ensures orphaned contacts are deleted

---

### domains

Core table for domain registrations.

```sql
CREATE TABLE domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    domain_name TEXT NOT NULL UNIQUE,
    tld TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN (
        'pending', 'active', 'expired', 'grace', 'redemption',
        'pending_delete', 'transferred_away', 'cancelled'
    )),
    auto_renew INTEGER NOT NULL DEFAULT 1 CHECK(auto_renew IN (0, 1)),
    is_locked INTEGER NOT NULL DEFAULT 1 CHECK(is_locked IN (0, 1)),
    whois_privacy INTEGER NOT NULL DEFAULT 0 CHECK(whois_privacy IN (0, 1)),
    auth_code TEXT,
    opensrs_domain_id TEXT UNIQUE,
    registered_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    renewed_at TEXT,
    transferred_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT
);

CREATE INDEX idx_domains_customer_id ON domains(customer_id);
CREATE INDEX idx_domains_domain_name ON domains(domain_name);
CREATE INDEX idx_domains_status ON domains(status);
CREATE INDEX idx_domains_expires_at ON domains(expires_at);
CREATE INDEX idx_domains_tld ON domains(tld);
CREATE INDEX idx_domains_opensrs_id ON domains(opensrs_domain_id);
```

**Normalization Notes:**
- domain_name is unique across the system
- Boolean fields use INTEGER with CHECK constraint (SQLite standard)
- Foreign key RESTRICT prevents deleting customers with domains
- opensrs_domain_id caches the external system's identifier
- TLD extracted for querying and reporting purposes

---

### domain_contacts

Junction table linking domains to contacts with specific roles.

```sql
CREATE TABLE domain_contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    contact_id INTEGER NOT NULL,
    contact_role TEXT NOT NULL CHECK(contact_role IN ('registrant', 'admin', 'technical', 'billing')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE RESTRICT,
    UNIQUE(domain_id, contact_role)
);

CREATE INDEX idx_domain_contacts_domain_id ON domain_contacts(domain_id);
CREATE INDEX idx_domain_contacts_contact_id ON domain_contacts(contact_id);
CREATE INDEX idx_domain_contacts_role ON domain_contacts(contact_role);
```

**Normalization Notes:**
- Many-to-many relationship between domains and contacts
- UNIQUE constraint ensures one contact per role per domain
- Allows contact reuse across multiple domains
- Cascade delete when domain is removed, restrict when contact is removed

---

### nameservers

Stores nameserver configurations for domains.

```sql
CREATE TABLE nameservers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    hostname TEXT NOT NULL,
    ip_address TEXT,
    sort_order INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE
);

CREATE INDEX idx_nameservers_domain_id ON nameservers(domain_id);
CREATE INDEX idx_nameservers_hostname ON nameservers(hostname);
```

**Normalization Notes:**
- One-to-many relationship (domain has multiple nameservers)
- sort_order preserves nameserver priority
- IP address optional (for glue records)
- Cascade delete when domain is removed

---

### invoices

Billing invoices for customers.

```sql
CREATE TABLE invoices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    invoice_number TEXT NOT NULL UNIQUE,
    invoice_date TEXT NOT NULL DEFAULT (datetime('now')),
    due_date TEXT NOT NULL,
    subtotal REAL NOT NULL DEFAULT 0.0,
    tax_amount REAL NOT NULL DEFAULT 0.0,
    total_amount REAL NOT NULL DEFAULT 0.0,
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN (
        'draft', 'sent', 'paid', 'overdue', 'cancelled', 'refunded'
    )),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT
);

CREATE INDEX idx_invoices_customer_id ON invoices(customer_id);
CREATE INDEX idx_invoices_invoice_number ON invoices(invoice_number);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_date ON invoices(due_date);
```

**Normalization Notes:**
- invoice_number is unique for external reference
- Separate subtotal, tax, and total for accounting accuracy
- Status tracks invoice lifecycle
- Foreign key RESTRICT prevents deleting customers with invoices

---

### billing_items

Line items for invoices (domain registrations, renewals, transfers).

```sql
CREATE TABLE billing_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL,
    domain_id INTEGER,
    item_type TEXT NOT NULL CHECK(item_type IN (
        'registration', 'renewal', 'transfer', 'whois_privacy', 'other'
    )),
    description TEXT NOT NULL,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price REAL NOT NULL,
    total_price REAL NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE SET NULL
);

CREATE INDEX idx_billing_items_invoice_id ON billing_items(invoice_id);
CREATE INDEX idx_billing_items_domain_id ON billing_items(domain_id);
CREATE INDEX idx_billing_items_item_type ON billing_items(item_type);
```

**Normalization Notes:**
- One-to-many relationship with invoices
- domain_id is nullable for non-domain charges
- SET NULL on domain deletion preserves invoice history
- item_type categorizes charges for reporting

---

### payments

Payment transactions for invoices.

```sql
CREATE TABLE payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id INTEGER NOT NULL,
    customer_id INTEGER NOT NULL,
    payment_method TEXT NOT NULL CHECK(payment_method IN (
        'credit_card', 'bank_transfer', 'paypal', 'credit', 'other'
    )),
    amount REAL NOT NULL,
    transaction_id TEXT,
    payment_date TEXT NOT NULL DEFAULT (datetime('now')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN (
        'pending', 'completed', 'failed', 'refunded'
    )),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE RESTRICT,
    FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT
);

CREATE INDEX idx_payments_invoice_id ON payments(invoice_id);
CREATE INDEX idx_payments_customer_id ON payments(customer_id);
CREATE INDEX idx_payments_payment_date ON payments(payment_date);
CREATE INDEX idx_payments_status ON payments(status);
```

**Normalization Notes:**
- Multiple payments can be applied to one invoice (partial payments)
- transaction_id stores external payment processor reference
- Status tracks payment lifecycle
- Foreign key RESTRICT preserves financial records

---

### tld_data

Registry-specific data requirements (e.g., .AU eligibility, .CA language preference).

```sql
CREATE TABLE tld_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    tld TEXT NOT NULL,
    data_key TEXT NOT NULL,
    data_value TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    UNIQUE(domain_id, data_key)
);

CREATE INDEX idx_tld_data_domain_id ON tld_data(domain_id);
CREATE INDEX idx_tld_data_tld ON tld_data(tld);
CREATE INDEX idx_tld_data_key ON tld_data(data_key);
```

**Normalization Notes:**
- Key-value store for flexible registry requirements
- UNIQUE constraint prevents duplicate keys per domain
- tld field allows querying requirements by TLD
- Cascade delete when domain is removed

---

### audit_log

Complete audit trail of all changes to domain-related records.

```sql
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    record_id INTEGER NOT NULL,
    operation TEXT NOT NULL CHECK(operation IN ('INSERT', 'UPDATE', 'DELETE')),
    old_values TEXT,
    new_values TEXT,
    changed_by TEXT NOT NULL,
    changed_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip_address TEXT,
    user_agent TEXT
);

CREATE INDEX idx_audit_log_table_record ON audit_log(table_name, record_id);
CREATE INDEX idx_audit_log_changed_at ON audit_log(changed_at);
CREATE INDEX idx_audit_log_changed_by ON audit_log(changed_by);
```

**Normalization Notes:**
- Generic audit table for all domain changes
- old_values and new_values store JSON snapshots
- changed_by references user/customer identifier
- IP address and user agent for security auditing
- No foreign keys to prevent deletion cascades

---

## Normalization Analysis

### First Normal Form (1NF)
✅ **Satisfied**
- All tables have primary keys
- All columns contain atomic values (no arrays or nested structures)
- Each column contains values of a single type
- No repeating groups or columns

### Second Normal Form (2NF)
✅ **Satisfied**
- All tables are in 1NF
- All non-key attributes are fully functionally dependent on the primary key
- No partial dependencies (relevant for composite keys)
- Junction tables (domain_contacts) use composite uniqueness but have surrogate primary keys

### Third Normal Form (3NF)
✅ **Satisfied**
- All tables are in 2NF
- No transitive dependencies
- Examples:
  - TLD extracted to `domains.tld` instead of parsing from domain_name
  - Contact information stored separately from domains
  - Invoice totals calculated from billing_items but denormalized for performance
  - Customer balance stored separately from invoice/payment calculations

### Denormalization Decisions

The following denormalizations are intentional for performance:

1. **invoices.total_amount** - Denormalized sum of billing_items for query performance
2. **customers.account_balance** - Denormalized for quick balance checks
3. **domains.tld** - Extracted from domain_name to avoid parsing on every query

These should be maintained via application logic or database triggers.

---

## SQLite-Specific Features

### Data Types
- **INTEGER** - For IDs, boolean flags (0/1), quantities
- **TEXT** - For strings, ISO 8601 timestamps
- **REAL** - For currency amounts (consider using INTEGER with fixed-point in production)

### Timestamp Handling
- All timestamps use ISO 8601 format via `datetime('now')`
- Example: `2026-01-15T06:25:00Z`
- Enables proper date sorting and filtering

### Boolean Values
- SQLite doesn't have native boolean type
- Use INTEGER with CHECK constraint: `CHECK(field IN (0, 1))`
- 0 = false, 1 = true

### Foreign Key Constraints
- Must be enabled in SQLite: `PRAGMA foreign_keys = ON;`
- Ensures referential integrity
- Supports CASCADE, RESTRICT, SET NULL actions

---

## Sea-ORM Implementation Notes

For implementation with sea-orm (Rust ORM):

1. **Entity Generation**: Use `sea-orm-cli generate entity` to create Rust entities from schema
2. **Migrations**: Create migration files in `migration/src/` directory
3. **Timestamps**: Use `DateTime<Utc>` type with automatic update on modification
4. **Enums**: Map CHECK constraints to Rust enums for type safety
5. **Relations**: Define relationships using `Related` and `Relation` traits
6. **Active Models**: Use active models for INSERT/UPDATE operations
7. **Query Optimization**: Use `select_only()` and `column()` for large tables

### Example sea-orm Entity (domains)

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "domains")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub customer_id: i32,
    pub domain_name: String,
    pub tld: String,
    pub status: DomainStatus,
    pub auto_renew: bool,
    pub is_locked: bool,
    pub whois_privacy: bool,
    pub auth_code: Option<String>,
    pub opensrs_domain_id: Option<String>,
    pub registered_at: DateTime,
    pub expires_at: DateTime,
    pub renewed_at: Option<DateTime>,
    pub transferred_at: Option<DateTime>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(None)")]
pub enum DomainStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "expired")]
    Expired,
    #[sea_orm(string_value = "grace")]
    Grace,
    #[sea_orm(string_value = "redemption")]
    Redemption,
    #[sea_orm(string_value = "pending_delete")]
    PendingDelete,
    #[sea_orm(string_value = "transferred_away")]
    TransferredAway,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::customers::Entity",
        from = "Column::CustomerId",
        to = "super::customers::Column::Id"
    )]
    Customer,
    #[sea_orm(has_many = "super::domain_contacts::Entity")]
    DomainContacts,
    #[sea_orm(has_many = "super::nameservers::Entity")]
    Nameservers,
    #[sea_orm(has_many = "super::tld_data::Entity")]
    TldData,
}
```

---

## Migration Strategy

### Initial Schema Setup

1. Create migration: `sea-orm-cli migrate generate init_schema`
2. Apply migration: `sea-orm-cli migrate up`
3. Generate entities: `sea-orm-cli generate entity -o src/entities`

### Schema Versioning

- Use sea-orm migrations for version control
- Each schema change gets a new migration file
- Migrations should be reversible (implement `down()`)
- Test migrations on copy of production data

---

## Performance Considerations

### Indexes
- Primary keys automatically indexed
- Foreign keys should have indexes (created above)
- Add composite indexes for common query patterns:
  ```sql
  CREATE INDEX idx_domains_customer_status ON domains(customer_id, status);
  CREATE INDEX idx_invoices_customer_status ON invoices(customer_id, status);
  ```

### Query Optimization
- Use EXPLAIN QUERY PLAN for slow queries
- Consider materialized views for complex reports
- Batch INSERT operations for bulk data
- Use transactions for related operations

### Maintenance
- `VACUUM` periodically to reclaim space
- `ANALYZE` to update query planner statistics
- Consider `WAL mode` for better concurrency

---

## Security Considerations

1. **Prepared Statements**: sea-orm handles this automatically
2. **Input Validation**: Validate all user inputs before database operations
3. **Access Control**: Implement row-level security via application logic
4. **Encryption**: Consider encrypting sensitive fields (auth_code, contact details)
5. **Audit Logging**: All domain changes tracked in audit_log table
6. **Backup Strategy**: Regular SQLite backups using `.backup` command or file copy

---

## Future Enhancements

Potential schema additions for future requirements:

1. **DNS Records Table** - Store DNS zone records if managing DNS
2. **Domain Renewals Queue** - Automated renewal processing
3. **Price History** - Track TLD pricing over time
4. **Customer API Keys** - For customer API access
5. **Notification Preferences** - Email notification settings
6. **Support Tickets** - Customer support integration
7. **Bulk Operation Batches** - Track bulk domain operations

---

## References

- [OpenSRS API Documentation](https://domains.opensrs.guide/)
- [Sea-ORM Documentation](https://www.sea-ql.org/SeaORM/)
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [Database Normalization Principles](https://en.wikipedia.org/wiki/Database_normalization)
