use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::Service as ProtoService;
use super::super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub service_endpoint: String,
}

#[cfg(test)]
impl Service {
    pub fn new(
        id: String,
        r#type: String,
        service_endpoint: String) -> Self {
        Service {
            id,
            r#type,
            service_endpoint
        }
    }
}

impl CheqdProtoBase for Service {
    type Proto = ProtoService;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            id: self.id.clone(),
            r#type: self.r#type.clone(),
            service_endpoint: self.service_endpoint.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            id: proto.id.clone(),
            r#type: proto.r#type.clone(),
            service_endpoint: proto.service_endpoint.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::Service;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_did_service() {
        let msg = Service::new(
            "id".into(),
            "type".into(),
            "service_endpoint".into()
        );

        let proto = msg.to_proto().unwrap();
        let decoded = Service::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
