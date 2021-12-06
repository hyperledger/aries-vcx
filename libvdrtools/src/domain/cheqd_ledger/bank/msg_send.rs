use cosmrs::proto::cosmos::bank::v1beta1::MsgSend as ProtoMsgSend;

use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;
use super::super::bank::Coin;

// MsgSend represents a message to send coins from one account to another.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct MsgSend {
    pub from_address: String,
    pub to_address: String,
    pub amount: Vec<Coin>,
}

impl MsgSend {
    pub fn new(
        from_address: String,
        to_address: String,
        amount: Vec<Coin>,
    ) -> Self {
        MsgSend {
            from_address,
            to_address,
            amount,
        }
    }
}

impl CheqdProtoBase for MsgSend {
    type Proto = ProtoMsgSend;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            from_address: self.from_address.clone(),
            to_address: self.to_address.clone(),
            amount: self.amount.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.from_address.clone(),
            proto.to_address.clone(),
            Vec::<Coin>::from_proto(&proto.amount)?,
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::environment;

    #[test]
    fn test_msg_send() {
        let coins = Coin::new(environment::cheqd_denom(), "100".to_string());
        let mut amount: Vec<Coin> = Vec::new();
        amount.push(coins);

        let msg = MsgSend::new(
            "cheqd1rnr5jrt4exl0samwj0yegv99jeskl0hsxmcz96".to_string(),
            "cheqd1rnr5jrt4exl0samwj0yegv99jeskl0hsxmcz96".to_string(),
            amount
        );

        let proto = msg.to_proto().unwrap();
        let decoded = MsgSend::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}