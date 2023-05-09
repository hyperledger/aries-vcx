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