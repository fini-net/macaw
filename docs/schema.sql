-- Macaw Domain Registration Database Schema
-- SQLite compatible schema with sea-orm support
-- Target: SQLite 3.35+

PRAGMA foreign_keys = ON;

-- ============================================================================
-- CUSTOMER MANAGEMENT
-- ============================================================================

-- Customers table - multi-customer support
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

CREATE INDEX idx_customers_username ON customers(username);
CREATE INDEX idx_customers_email ON customers(email);
CREATE INDEX idx_customers_status ON customers(status);

-- ============================================================================
-- CONTACT MANAGEMENT
-- ============================================================================

-- Contacts table - reusable contact information
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

CREATE INDEX idx_contacts_customer ON contacts(customer_id);
CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_type ON contacts(contact_type);

-- ============================================================================
-- DOMAIN MANAGEMENT
-- ============================================================================

-- Domains table - domain registrations
CREATE TABLE domains (
    domain_id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL,
    domain_name TEXT NOT NULL UNIQUE,
    tld TEXT NOT NULL,  -- extracted from domain_name for querying
    status TEXT NOT NULL CHECK(status IN (
        'pending', 'active', 'expired', 'grace', 'redemption',
        'pending_delete', 'transferred_away', 'cancelled'
    )) DEFAULT 'pending',

    -- Registration dates
    registration_date DATETIME NOT NULL,
    expiration_date DATETIME NOT NULL,
    auto_renew BOOLEAN NOT NULL DEFAULT 1,

    -- Transfer/security
    transfer_lock BOOLEAN NOT NULL DEFAULT 1,
    auth_code TEXT,  -- EPP auth code for transfers

    -- Privacy
    whois_privacy BOOLEAN NOT NULL DEFAULT 0,

    -- OpenSRS specific
    registry_domain_id TEXT,  -- Registry ID from OpenSRS

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (customer_id) REFERENCES customers(customer_id) ON DELETE RESTRICT
);

CREATE INDEX idx_domains_customer ON domains(customer_id);
CREATE INDEX idx_domains_name ON domains(domain_name);
CREATE INDEX idx_domains_tld ON domains(tld);
CREATE INDEX idx_domains_status ON domains(status);
CREATE INDEX idx_domains_expiration ON domains(expiration_date);

-- Domain contacts junction table
CREATE TABLE domain_contacts (
    domain_id INTEGER NOT NULL,
    contact_id INTEGER NOT NULL,
    contact_role TEXT NOT NULL CHECK(contact_role IN ('registrant', 'admin', 'tech', 'billing')),
    PRIMARY KEY (domain_id, contact_role),
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(contact_id) ON DELETE RESTRICT
);

CREATE INDEX idx_domain_contacts_domain ON domain_contacts(domain_id);
CREATE INDEX idx_domain_contacts_contact ON domain_contacts(contact_id);

-- ============================================================================
-- DNS MANAGEMENT
-- ============================================================================

-- Nameservers table
CREATE TABLE nameservers (
    nameserver_id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    hostname TEXT NOT NULL,
    ip_address TEXT,  -- Optional glue record
    priority INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE
);

CREATE INDEX idx_nameservers_domain ON nameservers(domain_id);

-- ============================================================================
-- REGISTRY METADATA
-- ============================================================================

-- TLD-specific data (key-value store for registry requirements)
CREATE TABLE tld_data (
    tld_data_id INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_id INTEGER NOT NULL,
    data_key TEXT NOT NULL,
    data_value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (domain_id) REFERENCES domains(domain_id) ON DELETE CASCADE
);

CREATE INDEX idx_tld_data_domain ON tld_data(domain_id);
CREATE INDEX idx_tld_data_key ON tld_data(data_key);

-- ============================================================================
-- BILLING
-- ============================================================================

-- Invoices table
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

CREATE INDEX idx_invoices_customer ON invoices(customer_id);
CREATE INDEX idx_invoices_number ON invoices(invoice_number);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_date ON invoices(due_date);

-- Invoice line items
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

CREATE INDEX idx_billing_items_invoice ON billing_items(invoice_id);
CREATE INDEX idx_billing_items_domain ON billing_items(domain_id);

-- Payments table
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

CREATE INDEX idx_payments_customer ON payments(customer_id);
CREATE INDEX idx_payments_invoice ON payments(invoice_id);
CREATE INDEX idx_payments_transaction ON payments(transaction_id);
CREATE INDEX idx_payments_status ON payments(status);

-- ============================================================================
-- AUDIT TRAIL
-- ============================================================================

-- Audit log for all domain changes
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

CREATE INDEX idx_audit_table_record ON audit_log(table_name, record_id);
CREATE INDEX idx_audit_changed_by ON audit_log(changed_by);
CREATE INDEX idx_audit_timestamp ON audit_log(timestamp);

-- ============================================================================
-- SAMPLE DATA (commented out - uncomment to populate test data)
-- ============================================================================

/*
-- Sample customer
INSERT INTO customers (username, email, company_name, status)
VALUES ('testuser', 'test@example.com', 'Test Company', 'active');

-- Sample contact
INSERT INTO contacts (customer_id, contact_type, first_name, last_name, email, phone,
    address1, city, state_province, postal_code, country_code)
VALUES (1, 'registrant', 'John', 'Doe', 'john@example.com', '+1.4155551234',
    '123 Main St', 'San Francisco', 'CA', '94105', 'US');

-- Sample domain
INSERT INTO domains (customer_id, domain_name, tld, status, registration_date, expiration_date)
VALUES (1, 'example.com', 'com', 'active', '2024-01-01', '2025-01-01');

-- Link domain to contact
INSERT INTO domain_contacts (domain_id, contact_id, contact_role)
VALUES (1, 1, 'registrant');

-- Sample nameservers
INSERT INTO nameservers (domain_id, hostname, priority) VALUES
    (1, 'ns1.example.com', 1),
    (1, 'ns2.example.com', 2);
*/
