use hex;
use md5::{Digest, Md5};

/// Generate OpenSRS signature: md5(md5(xml + api_key) + api_key)
///
/// OpenSRS requires lowercase hex output (the hex crate already does this).
pub fn generate_signature(xml_content: &str, api_key: &str) -> String {
    // Step 1: md5(xml + api_key)
    let mut hasher = Md5::new();
    hasher.update(xml_content.as_bytes());
    hasher.update(api_key.as_bytes());
    let first_hash = hasher.finalize();
    let first_hex = hex::encode(first_hash);

    // Step 2: md5(first_hash + api_key)
    let mut hasher = Md5::new();
    hasher.update(first_hex.as_bytes());
    hasher.update(api_key.as_bytes());
    let final_hash = hasher.finalize();
    hex::encode(final_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        let xml = r#"<?xml version='1.0' encoding='UTF-8' standalone='no' ?>
<!DOCTYPE OPS_envelope SYSTEM 'ops.dtd'>
<OPS_envelope><header><version>0.9</version></header></OPS_envelope>"#;
        let key = "test_key_12345";

        let sig = generate_signature(xml, key);

        // Verify it's 32 character hex string
        assert_eq!(sig.len(), 32);
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
        // Must be lowercase
        assert_eq!(sig, sig.to_lowercase());
    }

    #[test]
    fn test_signature_is_deterministic() {
        let xml = "test content";
        let key = "test_key";

        let sig1 = generate_signature(xml, key);
        let sig2 = generate_signature(xml, key);

        assert_eq!(sig1, sig2);
    }
}
