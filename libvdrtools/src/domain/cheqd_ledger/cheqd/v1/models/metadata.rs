use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::Metadata as ProtoMetadata;
use super::super::super::super::CheqdProtoBase;
use indy_api_types::errors::IndyResult;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub created: String,
    pub updated: String,
    pub deactivated: bool,
    pub version_id: String,
}

#[cfg(test)]
impl Metadata {
    pub fn new(created: String,
               updated: String,
               deactivated: bool,
               version_id:String) -> Self {
        Metadata {
            created,
            updated,
            deactivated,
            version_id
        }
    }
}

impl CheqdProtoBase for Metadata {
    type Proto = ProtoMetadata;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            created: self.created.clone(),
            updated: self.updated.clone(),
            deactivated: self.deactivated.clone(),
            version_id: self.version_id.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            created: proto.created.clone(),
            updated: proto.updated.clone(),
            deactivated: proto.deactivated.clone(),
            version_id: proto.version_id.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::Metadata;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_metadata_struct() {
        let msg = Metadata::new(
            "created".into(),
            "updated".into(),
            true,
            "version_id".into());

        let proto = msg.to_proto().unwrap();
        let decoded = Metadata::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
