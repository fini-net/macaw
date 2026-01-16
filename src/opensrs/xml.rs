use super::error::{OpenSrsError, Result};
use super::types::*;

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
    #[allow(unused)]
    let mut in_attributes = false;
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
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"key" {
                                current_key = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if !in_data_block {
                    continue;
                }

                let text = e.unescape().unwrap_or_default().trim().to_string();
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
                    "attributes" => in_attributes = true,
                    "exp_domains" => in_exp_domains = true,
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

/// Calculate Content-Length (OpenSRS requires exact byte count)
pub fn calculate_content_length(xml: &str) -> usize {
    xml.as_bytes().len()
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
