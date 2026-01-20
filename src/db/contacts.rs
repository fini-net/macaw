use crate::entities::{
    contacts, domain_contacts,
    prelude::{Contacts, DomainContacts},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};

/// Contact information structure
#[derive(Debug, Clone)]
pub struct ContactInfo {
    pub contact_type: String,
    pub first_name: String,
    pub last_name: String,
    pub organization: Option<String>,
    pub email: String,
    pub phone: String,
    pub fax: Option<String>,
    pub address1: String,
    pub address2: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country_code: String,
}

/// Find or create a contact with exact-match deduplication
pub async fn find_or_create_contact(
    db: &DatabaseConnection,
    customer_id: i32,
    contact_info: &ContactInfo,
) -> Result<i32, DbErr> {
    // Try to find exact match for this customer
    let existing = Contacts::find()
        .filter(contacts::Column::CustomerId.eq(customer_id))
        .filter(contacts::Column::FirstName.eq(&contact_info.first_name))
        .filter(contacts::Column::LastName.eq(&contact_info.last_name))
        .filter(contacts::Column::Email.eq(&contact_info.email))
        .filter(contacts::Column::Phone.eq(&contact_info.phone))
        .filter(contacts::Column::Address1.eq(&contact_info.address1))
        .filter(contacts::Column::City.eq(&contact_info.city))
        .filter(contacts::Column::StateProvince.eq(&contact_info.state_province))
        .filter(contacts::Column::PostalCode.eq(&contact_info.postal_code))
        .filter(contacts::Column::CountryCode.eq(&contact_info.country_code))
        .one(db)
        .await?;

    if let Some(contact) = existing {
        // Check optional fields for exact match
        let org_matches = match (&contact.organization, &contact_info.organization) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        };

        let fax_matches = match (&contact.fax, &contact_info.fax) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        };

        let addr2_matches = match (&contact.address2, &contact_info.address2) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        };

        if org_matches && fax_matches && addr2_matches {
            return Ok(contact.contact_id);
        }
    }

    // No exact match found, create new contact
    let new_contact = contacts::ActiveModel {
        customer_id: Set(customer_id),
        contact_type: Set(contact_info.contact_type.clone()),
        first_name: Set(contact_info.first_name.clone()),
        last_name: Set(contact_info.last_name.clone()),
        organization: Set(contact_info.organization.clone()),
        email: Set(contact_info.email.clone()),
        phone: Set(contact_info.phone.clone()),
        fax: Set(contact_info.fax.clone()),
        address1: Set(contact_info.address1.clone()),
        address2: Set(contact_info.address2.clone()),
        city: Set(contact_info.city.clone()),
        state_province: Set(contact_info.state_province.clone()),
        postal_code: Set(contact_info.postal_code.clone()),
        country_code: Set(contact_info.country_code.clone()),
        created_at: Set(chrono::Utc::now().to_rfc3339()),
        updated_at: Set(chrono::Utc::now().to_rfc3339()),
        ..Default::default()
    };

    let contact = new_contact.insert(db).await?;
    Ok(contact.contact_id)
}

/// Link a contact to a domain with a specific role
pub async fn link_contact_to_domain(
    db: &DatabaseConnection,
    domain_id: i32,
    contact_id: i32,
    contact_role: &str,
) -> Result<(), DbErr> {
    // Check if link already exists
    let existing = DomainContacts::find()
        .filter(domain_contacts::Column::DomainId.eq(domain_id))
        .filter(domain_contacts::Column::ContactRole.eq(contact_role))
        .one(db)
        .await?;

    if existing.is_some() {
        // Link already exists, skip
        return Ok(());
    }

    // Create new link
    let new_link = domain_contacts::ActiveModel {
        domain_id: Set(domain_id),
        contact_id: Set(contact_id),
        contact_role: Set(contact_role.to_string()),
    };

    new_link.insert(db).await?;
    Ok(())
}
