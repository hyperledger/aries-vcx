use cosmrs::proto::cosmos::base::query::v1beta1::PageRequest as ProtoPageRequest;
use indy_api_types::errors::IndyResult;

use super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PageRequest {
    pub key: Vec<u8>,
    pub offset: u64,
    pub limit: u64,
    pub count_total: bool,
}

impl PageRequest {
    pub fn new(key: Vec<u8>, offset: u64, limit: u64, count_total: bool) -> Self {
        PageRequest {
            key,
            offset,
            limit,
            count_total,
        }
    }
}

impl CheqdProtoBase for PageRequest {
    type Proto = ProtoPageRequest;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            key: self.key.clone(),
            offset: self.offset.clone(),
            limit: self.limit.clone(),
            count_total: self.count_total.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.key.clone(),
            proto.offset.clone(),
            proto.limit.clone(),
            proto.count_total.clone(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_msg_create_nym_request() {
        let msg = PageRequest::new(vec![0], 0, 3, false);

        let proto = msg.to_proto().unwrap();
        let decoded = PageRequest::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
