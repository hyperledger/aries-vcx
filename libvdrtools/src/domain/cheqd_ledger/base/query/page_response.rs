use cosmrs::proto::cosmos::base::query::v1beta1::PageResponse as ProtoPageResponse;
use indy_api_types::errors::IndyResult;

use super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct PageResponse {
    pub next_key: Vec<u8>,
    pub total: u64,
}

impl PageResponse {
    pub fn new(next_key: Vec<u8>, total: u64) -> Self {
        PageResponse { next_key, total }
    }
}

impl CheqdProtoBase for PageResponse {
    type Proto = ProtoPageResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            next_key: self.next_key.clone(),
            total: self.total.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(proto.next_key.clone(), proto.total.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_msg_create_nym_response() {
        let msg = PageResponse::new(vec![0], 1);

        let proto = msg.to_proto().unwrap();
        let decoded = PageResponse::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
