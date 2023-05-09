//! Module containing the actual messages data structures, apart from the `@type` field which is
//! handled through types in [`crate::msg_types`].

pub mod protocols;
pub mod traits;

pub mod common {
    pub mod attr_value {
        #[derive(Debug, Serialize, Deserialize)]
        #[serde(rename_all = "lowercase")]
        #[serde(untagged)]
        pub enum AttrValue {
            Encoded(EncodedAttrValue),
            Plain(PlainAttrValue),
        }
        
        #[derive(Debug, Serialize, Deserialize)]
        pub struct PlainAttrValue {
            value: String,
        }
        
        #[derive(Debug, Serialize, Deserialize)]
        pub struct EncodedAttrValue {
            value: String,
            #[serde(rename = "mime-type")]
            mime_type: Option<MaybeKnown<MimeType>>,
        }
    }
}