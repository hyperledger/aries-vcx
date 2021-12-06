use cosmrs::proto::cosmos::auth::v1beta1::BaseAccount as ProtoBaseAccount;
use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;
use super::super::crypto::PubKey;

/// BaseAccount defines a base account type. It contains all the necessary fields
/// for basic account functionality. Any custom account type should extend this
/// type for additional functionality (e.g. vesting).
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct BaseAccount {
    pub address: String,
    pub pub_key: Option<PubKey>,
    pub account_number: u64,
    pub sequence: u64,
}

impl BaseAccount {
    pub fn new(
        address: String,
        pub_key: Option<PubKey>,
        account_number: u64,
        sequence: u64,
    ) -> Self {
        BaseAccount {
            address,
            pub_key,
            account_number,
            sequence,
        }
    }
}

impl CheqdProtoBase for BaseAccount {
    type Proto = ProtoBaseAccount;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
         Ok(Self::Proto {
                address: self.address.clone(),
                pub_key: self.pub_key.to_proto()?,
                account_number: self.account_number,
                sequence: self.sequence,
            })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.address.clone(),
            Option::<PubKey>::from_proto(&proto.pub_key)?,
            proto.account_number,
            proto.sequence,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::super::QueryAccountRequest;

    use super::*;

    #[test]
    fn test_query_account_request() {
        let msg =
            QueryAccountRequest::new("cheqd1rnr5jrt4exl0samwj0yegv99jeskl0hsxmcz96".to_string());

        let proto = msg.to_proto().unwrap();
        let decoded = QueryAccountRequest::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
