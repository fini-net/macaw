# Entity Relationship Diagram

This document provides the ERD for the Macaw domain registration database.

## Mermaid ERD

```mermaid
erDiagram
    customers ||--o{ contacts : "has"
    customers ||--o{ domains : "owns"
    customers ||--o{ invoices : "receives"
    customers ||--o{ payments : "makes"

    domains ||--o{ domain_contacts : "has"
    contacts ||--o{ domain_contacts : "assigned_to"
    domains ||--o{ nameservers : "uses"
    domains ||--o{ tld_data : "has"
    domains ||--o{ billing_items : "billed_for"

    invoices ||--o{ billing_items : "contains"
    invoices ||--o{ payments : "paid_by"

    customers {
        INTEGER customer_id PK
        TEXT username UK "Authelia username"
        TEXT email
        TEXT company_name
        DECIMAL account_balance
        DECIMAL credit_limit
        TEXT status "active, suspended, closed"
        DATETIME created_at
        DATETIME updated_at
    }

    contacts {
        INTEGER contact_id PK
        INTEGER customer_id FK
        TEXT contact_type "registrant, admin, tech, billing"
        TEXT first_name
        TEXT last_name
        TEXT organization
        TEXT email
        TEXT phone
        TEXT fax
        TEXT address1
        TEXT address2
        TEXT city
        TEXT state_province
        TEXT postal_code
        TEXT country_code "ISO 3166-1 alpha-2"
        DATETIME created_at
        DATETIME updated_at
    }

    domains {
        INTEGER domain_id PK
        INTEGER customer_id FK
        TEXT domain_name UK
        TEXT tld
        TEXT status "pending, active, expired, etc."
        DATETIME registration_date
        DATETIME expiration_date
        BOOLEAN auto_renew
        BOOLEAN transfer_lock
        TEXT auth_code
        BOOLEAN whois_privacy
        TEXT registry_domain_id
        DATETIME created_at
        DATETIME updated_at
    }

    domain_contacts {
        INTEGER domain_id PK_FK
        INTEGER contact_id FK
        TEXT contact_role PK "registrant, admin, tech, billing"
    }

    nameservers {
        INTEGER nameserver_id PK
        INTEGER domain_id FK
        TEXT hostname
        TEXT ip_address "Optional glue record"
        INTEGER priority
        DATETIME created_at
    }

    tld_data {
        INTEGER tld_data_id PK
        INTEGER domain_id FK
        TEXT data_key
        TEXT data_value
        DATETIME created_at
        DATETIME updated_at
    }

    invoices {
        INTEGER invoice_id PK
        INTEGER customer_id FK
        TEXT invoice_number UK
        DATETIME invoice_date
        DATETIME due_date
        DECIMAL total_amount
        DECIMAL paid_amount
        TEXT status "draft, issued, paid, overdue, cancelled"
        DATETIME created_at
        DATETIME updated_at
    }

    billing_items {
        INTEGER billing_item_id PK
        INTEGER invoice_id FK
        INTEGER domain_id FK "Optional"
        TEXT item_type "registration, renewal, transfer, privacy, other"
        TEXT description
        INTEGER quantity
        DECIMAL unit_price
        DECIMAL total_price
        DATETIME created_at
    }

    payments {
        INTEGER payment_id PK
        INTEGER customer_id FK
        INTEGER invoice_id FK "Optional"
        TEXT payment_method "credit_card, bank_transfer, paypal, credit, other"
        TEXT transaction_id
        DECIMAL amount
        DATETIME payment_date
        TEXT status "pending, completed, failed, refunded"
        TEXT notes
        DATETIME created_at
    }
```

## Relationship Details

### One-to-Many Relationships

1. **customers → contacts**
   - A customer can have multiple contacts
   - Each contact belongs to one customer
   - Delete behavior: CASCADE (deleting customer deletes contacts)

2. **customers → domains**
   - A customer can own multiple domains
   - Each domain belongs to one customer
   - Delete behavior: RESTRICT (cannot delete customer with active domains)

3. **customers → invoices**
   - A customer can have multiple invoices
   - Each invoice belongs to one customer
   - Delete behavior: RESTRICT (cannot delete customer with invoices)

4. **customers → payments**
   - A customer can make multiple payments
   - Each payment is from one customer
   - Delete behavior: RESTRICT (cannot delete customer with payments)

5. **domains → nameservers**
   - A domain can have multiple nameservers (typically 2-13)
   - Each nameserver belongs to one domain
   - Delete behavior: CASCADE (deleting domain deletes nameservers)

6. **domains → tld_data**
   - A domain can have multiple TLD-specific data entries
   - Each TLD data entry belongs to one domain
   - Delete behavior: CASCADE (deleting domain deletes TLD data)

7. **invoices → billing_items**
   - An invoice can have multiple line items
   - Each billing item belongs to one invoice
   - Delete behavior: CASCADE (deleting invoice deletes billing items)

8. **invoices → payments**
   - An invoice can have multiple payments (partial payments)
   - Each payment can optionally link to one invoice
   - Delete behavior: SET NULL (deleting invoice preserves payment record)

9. **domains → billing_items**
   - A domain can appear on multiple billing items (registration, renewals, etc.)
   - Each billing item can optionally reference one domain
   - Delete behavior: SET NULL (preserve billing history if domain deleted)

### Many-to-Many Relationships

1. **domains ↔ contacts** (via domain_contacts)
   - A domain has exactly 4 contact roles: registrant, admin, tech, billing
   - Each contact can be used by multiple domains in different roles
   - The same contact can fill multiple roles for a single domain
   - Delete behavior:
     - Deleting domain: CASCADE (remove all contact associations)
     - Deleting contact: RESTRICT (cannot delete contact in use)

### Audit Trail

The **audit_log** table tracks changes to all other tables but doesn't have formal foreign key relationships. It references records via `table_name` and `record_id` pairs.

## Indexes

All foreign key columns are indexed for performance:

- `customers`: username, email, status
- `contacts`: customer_id, email, contact_type
- `domains`: customer_id, domain_name (unique), tld, status, expiration_date
- `domain_contacts`: domain_id, contact_id
- `nameservers`: domain_id
- `tld_data`: domain_id, data_key
- `invoices`: customer_id, invoice_number (unique), status, due_date
- `billing_items`: invoice_id, domain_id
- `payments`: customer_id, invoice_id, transaction_id, status
- `audit_log`: (table_name, record_id), changed_by, timestamp

## Constraints

### Check Constraints

- **customers.status**: active, suspended, closed
- **contacts.contact_type**: registrant, admin, tech, billing
- **domains.status**: pending, active, expired, grace, redemption, pending_delete, transferred_away, cancelled
- **domain_contacts.contact_role**: registrant, admin, tech, billing
- **billing_items.item_type**: registration, renewal, transfer, privacy, other
- **invoices.status**: draft, issued, paid, overdue, cancelled
- **payments.payment_method**: credit_card, bank_transfer, paypal, credit, other
- **payments.status**: pending, completed, failed, refunded
- **audit_log.action**: INSERT, UPDATE, DELETE

### Unique Constraints

- **customers.username**: Each Authelia username must be unique
- **domains.domain_name**: Each domain can only be registered once
- **invoices.invoice_number**: Each invoice has a unique number
- **domain_contacts**: Composite primary key on (domain_id, contact_role) ensures each role is filled exactly once

## Normalization

This schema is in Third Normal Form (3NF):

- **1NF**: All columns contain atomic values, no repeating groups
- **2NF**: No partial dependencies (all non-key attributes depend on entire primary key)
- **3NF**: No transitive dependencies (all attributes depend only on primary key)

### Intentional Denormalizations

1. **domains.tld** - Extracted from domain_name for efficient querying by TLD
2. **invoices.total_amount** and **paid_amount** - Cached sums for performance (should be updated via triggers or application logic)
3. **customers.account_balance** - Cached balance for quick access

These denormalizations improve query performance and are acceptable trade-offs for a domain registration system.
