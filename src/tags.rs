use std::collections::HashMap;

const MAX_TAG_VALUE_LENGTH: usize = 256;
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),

    #[error("Invalid tag value: {name}={value}")]
    InvalidTagValue { name: String, value: String },
}

pub fn create_tags(
    custom_tags: HashMap<String, String>,
) -> Result<HashMap<String, String>, TagError> {
    let mut tags = HashMap::with_capacity(10);

    // Add default tags
    tags.insert("processor".into(), "Cyberpunk".into());
    tags.insert("timestamp".into(), chrono::Utc::now().to_rfc3339());
    tags.insert(
        "host".into(),
        gethostname::gethostname().to_string_lossy().into(),
    );
    tags.insert("version".into(), VERSION.into());

    // Add provided custom tags
    for (k, v) in custom_tags {
        // Simple validation
        if !k.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(TagError::InvalidTagName(format!("Invalid tag name: {}", k)));
        }
        if v.len() > MAX_TAG_VALUE_LENGTH {
            return Err(TagError::InvalidTagValue { name: k, value: v });
        }
        tags.insert(k, v);
    }

    Ok(tags)
}
