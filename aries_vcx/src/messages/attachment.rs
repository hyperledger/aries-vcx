use std::str::from_utf8;

use serde_json;

use crate::error::{VcxError, VcxErrorKind, VcxResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Attachments(pub Vec<Attachment>);

impl Attachments {
    pub fn new() -> Attachments {
        Attachments::default()
    }

    pub fn get(&self) -> Option<&Attachment> {
        self.0.get(0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add(&mut self, attachment: Attachment) {
        self.0.push(attachment);
    }

    pub fn add_json_attachment(&mut self, id: AttachmentId, json: serde_json::Value, encoding: AttachmentEncoding) -> VcxResult<()> {
        let json: Json = Json::new(id, json, encoding)?;
        self.add(Attachment::JSON(json));
        Ok(())
    }

    pub fn add_base64_encoded_json_attachment(&mut self, id: AttachmentId, json: serde_json::Value) -> VcxResult<()> {
        self.add_json_attachment(id, json, AttachmentEncoding::Base64) // TODO: AttachmentEncoding::Json does not seem to work
    }

    pub fn content(&self) -> VcxResult<String> {
        match self.get() {
            Some(Attachment::JSON(ref attach)) => attach.get_data(),
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, "Unsupported Attachment type"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mime-type")]
pub enum Attachment {
    #[serde(rename = "application/json")]
    JSON(Json),
    Blank,
}

impl Attachment {
    pub fn id(&self) -> Option<AttachmentId> {
        match self {
            Self::JSON(json) => Some(json.id.clone()),
            Self::Blank => None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Json {
    #[serde(rename = "@id")]
    id: AttachmentId,
    data: AttachmentData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttachmentId {
    #[serde(rename = "libindy-cred-offer-0")]
    CredentialOffer,
    #[serde(rename = "libindy-cred-request-0")]
    CredentialRequest,
    #[serde(rename = "libindy-cred-0")]
    Credential,
    #[serde(rename = "libindy-request-presentation-0")]
    PresentationRequest,
    #[serde(rename = "libindy-presentation-0")]
    Presentation,
}

impl Json {
    pub fn new(id: AttachmentId, json: serde_json::Value, encoding: AttachmentEncoding) -> VcxResult<Json> {
        let data: AttachmentData = match encoding {
            AttachmentEncoding::Base64 => {
                AttachmentData::Base64(
                    base64::encode(&
                        match json {
                            serde_json::Value::Object(obj) => {
                                serde_json::to_string(&obj)
                                    .map_err(|_| VcxError::from_msg(VcxErrorKind::InvalidJson, "Invalid Attachment Json".to_string()))?
                            }
                            serde_json::Value::String(str) => str,
                            val => return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Unsupported Json value: {:?}", val)))
                        }
                    )
                )
            }
            AttachmentEncoding::Json => {
                AttachmentData::Json(json)
            }
        };
        Ok(Json {
            id,
            data,
        })
    }

    pub fn get_data(&self) -> VcxResult<String> {
        let data = self.data.get_bytes()?;
        trace!("Json::get_data >>> data: {:?}", data);
        from_utf8(data.as_slice())
            .map(|s| s.to_string())
            .map_err(|_| VcxError::from_msg(VcxErrorKind::IOError, "Wrong bytes in attachment".to_string()))
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AttachmentEncoding {
    Base64,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttachmentData {
    #[serde(rename = "base64")]
    Base64(String),
    #[serde(rename = "json")]
    Json(serde_json::Value),
}

impl AttachmentData {
    pub fn get_bytes(&self) -> VcxResult<Vec<u8>> {
        match self {
            AttachmentData::Base64(s) => {
                base64::decode(s).map_err(|_| VcxError::from_msg(VcxErrorKind::IOError, "Wrong bytes in attachment"))
            }
            AttachmentData::Json(json) => {
                serde_json::to_vec(&json).map_err(|_| VcxError::from_msg(VcxErrorKind::IOError, "Wrong bytes in attachment"))
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;

    fn _json() -> serde_json::Value {
        json!({"field": "value"})
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_json_attachment_works_base64() {
        let json_attachment: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
        assert_eq!(vec![123, 34, 102, 105, 101, 108, 100, 34, 58, 34, 118, 97, 108, 117, 101, 34, 125], json_attachment.data.get_bytes().unwrap());
        assert_eq!(_json().to_string(), json_attachment.get_data().unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_attachments_works_base64() {
        {
            let mut attachments = Attachments::new();
            assert_eq!(0, attachments.0.len());

            let json: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
            attachments.add(Attachment::JSON(json));
            assert_eq!(1, attachments.0.len());

            assert_eq!(_json().to_string(), attachments.content().unwrap());
        }

        {
            let mut attachments = Attachments::new();
            attachments.add_json_attachment(AttachmentId::Credential, _json(), AttachmentEncoding::Base64).unwrap();
            assert_eq!(_json().to_string(), attachments.content().unwrap());
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_json_attachment_works_json() {
        let json_attachment: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Json).unwrap();
        let bytes = json_attachment.data.get_bytes().unwrap();
        println!("{:?}", bytes);

        assert_eq!(vec![123, 34, 102, 105, 101, 108, 100, 34, 58, 34, 118, 97, 108, 117, 101, 34, 125], json_attachment.data.get_bytes().unwrap());
        assert_eq!(_json().to_string(), json_attachment.get_data().unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_attachments_works_json() {
        {
            let mut attachments = Attachments::new();
            assert_eq!(0, attachments.0.len());

            let json: Json = Json::new(AttachmentId::Credential, _json(), AttachmentEncoding::Json).unwrap();
            attachments.add(Attachment::JSON(json));
            assert_eq!(1, attachments.0.len());

            assert_eq!(_json().to_string(), attachments.content().unwrap());
        }

        {
            let mut attachments = Attachments::new();
            attachments.add_json_attachment(AttachmentId::Credential, _json(), AttachmentEncoding::Json).unwrap();
            assert_eq!(_json().to_string(), attachments.content().unwrap());
        }
    }
}
