-- Macaw Domain Registration Backend - SQLite Schema
-- Normalized database design for domain registration management
-- Compatible with SQLite 3.x and sea-orm

-- Enable foreign key constraints
PRAGMA foreign_keys = ON;

-- =============================================================================
-- CUSTOMERS
-- =============================================================================

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

-- =============================================================================
-- CONTACTS
-- =============================================================================

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

-- =============================================================================
-- DOMAINS
-- =============================================================================

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
CREATE INDEX idx_domains_customer_status ON domains(customer_id, status);

-- =============================================================================
-- DOMAIN CONTACTS (Junction Table)
-- =============================================================================

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

-- =============================================================================
-- NAMESERVERS
-- =============================================================================

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

-- =============================================================================
-- INVOICES
-- =============================================================================

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
CREATE INDEX idx_invoices_customer_status ON invoices(customer_id, status);

-- =============================================================================
-- BILLING ITEMS
-- =============================================================================

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

-- =============================================================================
-- PAYMENTS
-- =============================================================================

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

-- =============================================================================
-- TLD DATA (Registry-Specific Requirements)
-- =============================================================================

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

-- =============================================================================
-- AUDIT LOG
-- =============================================================================

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

-- =============================================================================
-- SAMPLE DATA (Optional - for testing)
-- =============================================================================

-- Uncomment to insert sample data for testing:
/*
INSERT INTO customers (email, company_name, status) VALUES
    ('admin@example.com', 'Example Corp', 'active');

INSERT INTO contacts (customer_id, type, first_name, last_name, email, phone, address1, city, postal_code, country_code) VALUES
    (1, 'registrant', 'John', 'Doe', 'john@example.com', '+1-555-1234', '123 Main St', 'New York', '10001', 'US');

INSERT INTO domains (customer_id, domain_name, tld, status, registered_at, expires_at) VALUES
    (1, 'example.com', 'com', 'active', datetime('now'), datetime('now', '+1 year'));
*/
