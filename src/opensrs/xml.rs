use super::error::{OpenSrsError, Result};
use super::types::*;

use quick_xml::escape::unescape as unescape_xml;

/// Decode and unescape a `BytesText` event into an owned, trimmed `String`.
///
/// `quick-xml` 0.41 removed `BytesText::unescape()`; the replacement is the
/// free function `quick_xml::escape::unescape`, which operates on a `&str`.
/// OpenSRS responses are UTF-8, so we decode the raw bytes and then unescape.
fn text_event_to_string(e: &quick_xml::events::BytesText<'_>) -> String {
    let decoded = e.decode().unwrap_or_default();
    unescape_xml(&decoded)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Serialize request to OpenSRS XML format
///
/// OpenSRS uses a non-standard XML structure with <dt_assoc> and <item key="..."> tags.
/// We'll use manual XML construction for now instead of fighting with serde.
pub fn serialize_request(request: &GetDomainsByExpireDateRequest) -> Result<String> {
    let mut xml = String::from(
        r#"<?xml version='1.0' encoding='UTF-8' standalone='no' ?>
<!DOCTYPE OPS_envelope SYSTEM 'ops.dtd'>
<OPS_envelope>
  <header>
    <version>0.9</version>
  </header>
  <body>
    <data_block>
      <dt_assoc>
        <item key="protocol">"#,
    );
    xml.push_str(&request.protocol);
    xml.push_str(
        r#"</item>
        <item key="object">"#,
    );
    xml.push_str(&request.object);
    xml.push_str(
        r#"</item>
        <item key="action">"#,
    );
    xml.push_str(&request.action);
    xml.push_str(
        r#"</item>
        <item key="attributes">
          <dt_assoc>
            <item key="exp_from">"#,
    );
    xml.push_str(&request.attributes.exp_from);
    xml.push_str(
        r#"</item>
            <item key="exp_to">"#,
    );
    xml.push_str(&request.attributes.exp_to);
    xml.push_str("</item>");

    if let Some(limit) = request.attributes.limit {
        xml.push_str(
            r#"
            <item key="limit">"#,
        );
        xml.push_str(&limit.to_string());
        xml.push_str("</item>");
    }

    if let Some(page) = request.attributes.page {
        xml.push_str(
            r#"
            <item key="page">"#,
        );
        xml.push_str(&page.to_string());
        xml.push_str("</item>");
    }

    xml.push_str(
        r#"
          </dt_assoc>
        </item>
      </dt_assoc>
    </data_block>
  </body>
</OPS_envelope>
"#,
    );

    Ok(xml)
}

/// Deserialize OpenSRS XML response
///
/// OpenSRS uses a dt_assoc/item structure that requires custom parsing.
pub fn deserialize_response(xml: &str) -> Result<GetDomainsByExpireDateResponse> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut is_success = false;
    let mut response_code = String::new();
    let mut response_text = String::new();
    let mut page = 0u32;
    let mut total = 0u32;
    let mut remainder = 0u8;
    let mut exp_domains = Vec::new();

    let mut current_key = String::new();
    let mut buf = Vec::new();

    // Simple state machine to track where we are in the XML
    let mut in_data_block = false;
    let mut in_exp_domains = false;
    let mut current_domain: Option<ExpiringDomain> = None;
    #[allow(unused)]
    let mut domain_field_key = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "data_block" {
                    in_data_block = true;
                } else if name == "item" {
                    // Extract key attribute
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"key" {
                            current_key = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if !in_data_block {
                    continue;
                }

                let text = text_event_to_string(&e);
                if text.is_empty() {
                    continue;
                }

                match current_key.as_str() {
                    "is_success" => {
                        is_success = text == "1" || text.to_lowercase() == "true";
                    }
                    "response_code" => response_code = text,
                    "response_text" => response_text = text,
                    "page" => page = text.parse().unwrap_or(0),
                    "total" => total = text.parse().unwrap_or(0),
                    "remainder" => remainder = text.parse().unwrap_or(0),
                    "name" if in_exp_domains => {
                        if let Some(ref mut domain) = current_domain {
                            domain.name = text;
                        } else {
                            current_domain = Some(ExpiringDomain {
                                name: text,
                                expiredate: String::new(),
                                f_auto_renew: String::new(),
                                f_let_expire: String::new(),
                            });
                        }
                    }
                    "expiredate" if in_exp_domains => {
                        if let Some(ref mut domain) = current_domain {
                            domain.expiredate = text;
                        }
                    }
                    "f_auto_renew" if in_exp_domains => {
                        if let Some(ref mut domain) = current_domain {
                            domain.f_auto_renew = text;
                        }
                    }
                    "f_let_expire" if in_exp_domains => {
                        if let Some(ref mut domain) = current_domain {
                            domain.f_let_expire = text;
                            // Domain complete, add to list
                            exp_domains.push(domain.clone());
                            current_domain = None;
                        }
                    }
                    "exp_domains" => in_exp_domains = true,
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "data_block" {
                    in_data_block = false;
                    in_exp_domains = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(OpenSrsError::XmlDeserialize(format!(
                    "XML parse error: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(GetDomainsByExpireDateResponse {
        is_success,
        response_code,
        response_text,
        attributes: GetDomainsByExpireDateResponseAttrs {
            page,
            total,
            remainder,
            exp_domains,
        },
    })
}

/// Serialize get_domain request to OpenSRS XML format
pub fn serialize_get_domain_request(request: &GetDomainRequest) -> Result<String> {
    let mut xml = String::from(
        r#"<?xml version='1.0' encoding='UTF-8' standalone='no' ?>
<!DOCTYPE OPS_envelope SYSTEM 'ops.dtd'>
<OPS_envelope>
  <header>
    <version>0.9</version>
  </header>
  <body>
    <data_block>
      <dt_assoc>
        <item key="protocol">"#,
    );
    xml.push_str(&request.protocol);
    xml.push_str(
        r#"</item>
        <item key="object">"#,
    );
    xml.push_str(&request.object);
    xml.push_str(
        r#"</item>
        <item key="action">"#,
    );
    xml.push_str(&request.action);
    xml.push_str(
        r#"</item>
        <item key="attributes">
          <dt_assoc>
            <item key="domain">"#,
    );
    xml.push_str(&request.attributes.domain);
    xml.push_str(
        r#"</item>
            <item key="type">"#,
    );
    xml.push_str(&request.attributes.req_type);
    xml.push_str(
        r#"</item>
          </dt_assoc>
        </item>
      </dt_assoc>
    </data_block>
  </body>
</OPS_envelope>
"#,
    );

    Ok(xml)
}

/// Serialize set_contact request to OpenSRS XML format
pub fn serialize_set_contact_request(request: &SetContactRequest) -> Result<String> {
    let mut xml = String::from(
        r#"<?xml version='1.0' encoding='UTF-8' standalone='no' ?>
<!DOCTYPE OPS_envelope SYSTEM 'ops.dtd'>
<OPS_envelope>
  <header>
    <version>0.9</version>
  </header>
  <body>
    <data_block>
      <dt_assoc>
        <item key="protocol">"#,
    );
    xml.push_str(&request.protocol);
    xml.push_str(
        r#"</item>
        <item key="object">"#,
    );
    xml.push_str(&request.object);
    xml.push_str(
        r#"</item>
        <item key="action">"#,
    );
    xml.push_str(&request.action);
    xml.push_str(
        r#"</item>
        <item key="attributes">
          <dt_assoc>
            <item key="domain">"#,
    );
    xml.push_str(&request.attributes.domain);
    xml.push_str(
        r#"</item>
            <item key="contact_set">
              <dt_assoc>
"#,
    );

    // Serialize each contact type if present
    if let Some(ref owner) = request.attributes.contact_set.owner {
        xml.push_str(
            r#"                <item key="owner">
                  <dt_assoc>
                    <item key="first_name">"#,
        );
        xml.push_str(&owner.first_name);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="last_name">"#,
        );
        xml.push_str(&owner.last_name);
        xml.push_str("</item>");
        if let Some(ref org) = owner.org_name {
            xml.push_str(
                r#"
                    <item key="org_name">"#,
            );
            xml.push_str(org);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="email">"#,
        );
        xml.push_str(&owner.email);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="phone">"#,
        );
        xml.push_str(&owner.phone);
        xml.push_str("</item>");
        if let Some(ref fax) = owner.fax {
            xml.push_str(
                r#"
                    <item key="fax">"#,
            );
            xml.push_str(fax);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="address1">"#,
        );
        xml.push_str(&owner.address1);
        xml.push_str("</item>");
        if let Some(ref addr2) = owner.address2 {
            xml.push_str(
                r#"
                    <item key="address2">"#,
            );
            xml.push_str(addr2);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="city">"#,
        );
        xml.push_str(&owner.city);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="state">"#,
        );
        xml.push_str(&owner.state);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="postal_code">"#,
        );
        xml.push_str(&owner.postal_code);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="country">"#,
        );
        xml.push_str(&owner.country);
        xml.push_str(
            r#"</item>
                  </dt_assoc>
                </item>
"#,
        );
    }

    if let Some(ref admin) = request.attributes.contact_set.admin {
        xml.push_str(
            r#"                <item key="admin">
                  <dt_assoc>
                    <item key="first_name">"#,
        );
        xml.push_str(&admin.first_name);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="last_name">"#,
        );
        xml.push_str(&admin.last_name);
        xml.push_str("</item>");
        if let Some(ref org) = admin.org_name {
            xml.push_str(
                r#"
                    <item key="org_name">"#,
            );
            xml.push_str(org);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="email">"#,
        );
        xml.push_str(&admin.email);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="phone">"#,
        );
        xml.push_str(&admin.phone);
        xml.push_str("</item>");
        if let Some(ref fax) = admin.fax {
            xml.push_str(
                r#"
                    <item key="fax">"#,
            );
            xml.push_str(fax);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="address1">"#,
        );
        xml.push_str(&admin.address1);
        xml.push_str("</item>");
        if let Some(ref addr2) = admin.address2 {
            xml.push_str(
                r#"
                    <item key="address2">"#,
            );
            xml.push_str(addr2);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="city">"#,
        );
        xml.push_str(&admin.city);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="state">"#,
        );
        xml.push_str(&admin.state);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="postal_code">"#,
        );
        xml.push_str(&admin.postal_code);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="country">"#,
        );
        xml.push_str(&admin.country);
        xml.push_str(
            r#"</item>
                  </dt_assoc>
                </item>
"#,
        );
    }

    if let Some(ref billing) = request.attributes.contact_set.billing {
        xml.push_str(
            r#"                <item key="billing">
                  <dt_assoc>
                    <item key="first_name">"#,
        );
        xml.push_str(&billing.first_name);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="last_name">"#,
        );
        xml.push_str(&billing.last_name);
        xml.push_str("</item>");
        if let Some(ref org) = billing.org_name {
            xml.push_str(
                r#"
                    <item key="org_name">"#,
            );
            xml.push_str(org);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="email">"#,
        );
        xml.push_str(&billing.email);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="phone">"#,
        );
        xml.push_str(&billing.phone);
        xml.push_str("</item>");
        if let Some(ref fax) = billing.fax {
            xml.push_str(
                r#"
                    <item key="fax">"#,
            );
            xml.push_str(fax);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="address1">"#,
        );
        xml.push_str(&billing.address1);
        xml.push_str("</item>");
        if let Some(ref addr2) = billing.address2 {
            xml.push_str(
                r#"
                    <item key="address2">"#,
            );
            xml.push_str(addr2);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="city">"#,
        );
        xml.push_str(&billing.city);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="state">"#,
        );
        xml.push_str(&billing.state);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="postal_code">"#,
        );
        xml.push_str(&billing.postal_code);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="country">"#,
        );
        xml.push_str(&billing.country);
        xml.push_str(
            r#"</item>
                  </dt_assoc>
                </item>
"#,
        );
    }

    if let Some(ref tech) = request.attributes.contact_set.tech {
        xml.push_str(
            r#"                <item key="tech">
                  <dt_assoc>
                    <item key="first_name">"#,
        );
        xml.push_str(&tech.first_name);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="last_name">"#,
        );
        xml.push_str(&tech.last_name);
        xml.push_str("</item>");
        if let Some(ref org) = tech.org_name {
            xml.push_str(
                r#"
                    <item key="org_name">"#,
            );
            xml.push_str(org);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="email">"#,
        );
        xml.push_str(&tech.email);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="phone">"#,
        );
        xml.push_str(&tech.phone);
        xml.push_str("</item>");
        if let Some(ref fax) = tech.fax {
            xml.push_str(
                r#"
                    <item key="fax">"#,
            );
            xml.push_str(fax);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="address1">"#,
        );
        xml.push_str(&tech.address1);
        xml.push_str("</item>");
        if let Some(ref addr2) = tech.address2 {
            xml.push_str(
                r#"
                    <item key="address2">"#,
            );
            xml.push_str(addr2);
            xml.push_str("</item>");
        }
        xml.push_str(
            r#"
                    <item key="city">"#,
        );
        xml.push_str(&tech.city);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="state">"#,
        );
        xml.push_str(&tech.state);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="postal_code">"#,
        );
        xml.push_str(&tech.postal_code);
        xml.push_str("</item>");
        xml.push_str(
            r#"
                    <item key="country">"#,
        );
        xml.push_str(&tech.country);
        xml.push_str(
            r#"</item>
                  </dt_assoc>
                </item>
"#,
        );
    }

    xml.push_str(
        r#"              </dt_assoc>
            </item>
          </dt_assoc>
        </item>
      </dt_assoc>
    </data_block>
  </body>
</OPS_envelope>
"#,
    );

    Ok(xml)
}

/// Deserialize OpenSRS get_domain all_info response
/// This is a simplified parser that extracts common fields
/// For a production system, you'd want more robust parsing
pub fn deserialize_domain_all_info(xml: &str) -> Result<GetDomainAllInfoResponse> {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    use std::collections::HashMap;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut is_success = false;
    let mut response_code = String::new();
    let mut response_text = String::new();

    // Domain attributes
    let mut domain_name = String::new();
    let mut domain_expdate = String::new();
    let mut registry_createdate = String::new();
    let mut auto_renew = String::new();
    let mut lock_state = String::new();
    let mut whois_privacy_state = String::new();
    let mut registry_domainid = String::new();

    // Contact sets
    let mut owner_contact: Option<ContactInfo> = None;
    let mut admin_contact: Option<ContactInfo> = None;
    let mut billing_contact: Option<ContactInfo> = None;
    let mut tech_contact: Option<ContactInfo> = None;

    // Track current contact being built
    let mut current_contact_type: Option<String> = None;
    let mut current_contact: Option<ContactInfo> = None;

    // Nameservers
    let nameserver_list: Vec<NameserverInfo> = Vec::new();

    // TLD data
    let tld_data_map: HashMap<String, String> = HashMap::new();

    let mut current_key = String::new();
    let mut buf = Vec::new();
    let mut in_data_block = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "data_block" {
                    in_data_block = true;
                } else if name == "item" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"key" {
                            let key = String::from_utf8_lossy(&attr.value).to_string();
                            current_key = key.clone();

                            // Check if entering a contact type
                            if key == "owner" || key == "admin" || key == "billing" || key == "tech"
                            {
                                current_contact_type = Some(key);
                                current_contact = Some(ContactInfo::default());
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if !in_data_block {
                    continue;
                }

                let text = text_event_to_string(&e);
                if text.is_empty() {
                    continue;
                }

                match current_key.as_str() {
                    "is_success" => {
                        is_success = text == "1" || text.to_lowercase() == "true";
                    }
                    "response_code" => response_code = text,
                    "response_text" => response_text = text,
                    "domain_name" => domain_name = text,
                    "domain_expdate" => domain_expdate = text,
                    "registry_createdate" => registry_createdate = text,
                    "auto_renew" => auto_renew = text,
                    "lock_state" => lock_state = text,
                    "whois_privacy_state" => whois_privacy_state = text,
                    "registry_domainid" => registry_domainid = text,
                    // Contact fields
                    "first_name" | "last_name" | "org_name" | "email" | "phone" | "fax"
                    | "address1" | "address2" | "city" | "state" | "postal_code" | "country" => {
                        if let Some(ref mut contact) = current_contact {
                            match current_key.as_str() {
                                "first_name" => contact.first_name = text,
                                "last_name" => contact.last_name = text,
                                "org_name" => contact.org_name = Some(text),
                                "email" => contact.email = text,
                                "phone" => contact.phone = text,
                                "fax" => contact.fax = Some(text),
                                "address1" => contact.address1 = text,
                                "address2" => contact.address2 = Some(text),
                                "city" => contact.city = text,
                                "state" => contact.state = text,
                                "postal_code" => contact.postal_code = text,
                                "country" => contact.country = text,
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "data_block" {
                    in_data_block = false;
                } else if name == "item" {
                    // Check if we're exiting a contact type
                    if let (Some(contact_type), Some(contact)) =
                        (current_contact_type.take(), current_contact.take())
                    {
                        let populated_contact = Some(contact);
                        match contact_type.as_str() {
                            "owner" => owner_contact = populated_contact,
                            "admin" => admin_contact = populated_contact,
                            "billing" => billing_contact = populated_contact,
                            "tech" => tech_contact = populated_contact,
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(OpenSrsError::XmlDeserialize(format!(
                    "XML parse error: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    let contact_set = ContactSet {
        owner: owner_contact,
        admin: admin_contact,
        billing: billing_contact,
        tech: tech_contact,
    };

    let tld_data = TldDataMap { data: tld_data_map };

    Ok(GetDomainAllInfoResponse {
        is_success,
        response_code,
        response_text,
        attributes: DomainAllInfo {
            domain_name,
            domain_expdate,
            registry_createdate,
            auto_renew,
            lock_state,
            whois_privacy_state,
            registry_domainid,
            contact_set,
            nameserver_list,
            tld_data,
        },
    })
}

/// Deserialize OpenSRS set_contact response
pub fn deserialize_set_contact_response(xml: &str) -> Result<SetContactResponse> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut is_success = false;
    let mut response_code = String::new();
    let mut response_text = String::new();

    let mut current_key = String::new();
    let mut buf = Vec::new();
    let mut in_data_block = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "data_block" {
                    in_data_block = true;
                } else if name == "item" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"key" {
                            current_key = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if !in_data_block {
                    continue;
                }

                let text = text_event_to_string(&e);
                if text.is_empty() {
                    continue;
                }

                match current_key.as_str() {
                    "is_success" => {
                        is_success = text == "1" || text.to_lowercase() == "true";
                    }
                    "response_code" => response_code = text,
                    "response_text" => response_text = text,
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "data_block" {
                    in_data_block = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(OpenSrsError::XmlDeserialize(format!(
                    "XML parse error: {}",
                    e
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(SetContactResponse {
        is_success,
        response_code,
        response_text,
    })
}

/// Calculate Content-Length (OpenSRS requires exact byte count)
#[allow(dead_code)]
pub fn calculate_content_length(xml: &str) -> usize {
    xml.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_basic_request() {
        let request = GetDomainsByExpireDateRequest {
            protocol: "XCP".to_string(),
            object: "DOMAIN".to_string(),
            action: "GET_DOMAINS_BY_EXPIREDATE".to_string(),
            attributes: GetDomainsByExpireDateAttrs {
                exp_from: "2026-01-01".to_string(),
                exp_to: "2026-12-31".to_string(),
                limit: None,
                page: None,
            },
        };

        let xml = serialize_request(&request).unwrap();

        assert!(xml.contains("<?xml version='1.0'"));
        assert!(xml.contains("<item key=\"protocol\">XCP</item>"));
        assert!(xml.contains("<item key=\"exp_from\">2026-01-01</item>"));
        assert!(xml.contains("<item key=\"exp_to\">2026-12-31</item>"));
    }

    #[test]
    fn test_content_length_is_bytes() {
        let xml = "test 测试";
        // "test " = 5 bytes, "测试" = 6 bytes (3 bytes per character)
        assert_eq!(calculate_content_length(xml), 11);
    }
}
