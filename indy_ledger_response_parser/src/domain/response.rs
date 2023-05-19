use indy_api_types::{
    errors::{err_msg, IndyErrorKind},
    IndyError,
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub req_id: u64,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Reply<T> {
    ReplyV0(ReplyV0<T>),
    ReplyV1(ReplyV1<T>),
}

impl<T> Reply<T> {
    pub fn result(self) -> T {
        match self {
            Reply::ReplyV0(reply) => reply.result,
            Reply::ReplyV1(mut reply) => reply.data.result.remove(0).result,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ReplyV0<T> {
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1<T> {
    pub data: ReplyDataV1<T>,
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1<T> {
    pub result: Vec<ReplyV0<T>>,
}

#[derive(Debug, Deserialize)]
pub struct GetReplyResultV0<T> {
    pub data: Option<T>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetReplyResultV1<T> {
    pub txn: GetReplyTxnV1<T>,
    pub txn_metadata: TxnMetadata,
}

#[derive(Debug, Deserialize)]
pub struct GetReplyTxnV1<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TxnMetadata {
    pub seq_no: u32,
    pub creation_time: u64,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum Message<T> {
    #[serde(rename = "REQNACK")]
    ReqNACK(Response),
    #[serde(rename = "REPLY")]
    Reply(Reply<T>),
    #[serde(rename = "REJECT")]
    Reject(Response),
}

pub trait ReplyType {
    fn get_type<'a>() -> &'a str;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum MessageWithTypedReply<'a, T> {
    #[serde(rename = "REQNACK")]
    ReqNACK(Response),
    #[serde(borrow)]
    #[serde(rename = "REPLY")]
    Reply(Reply<TypedReply<'a, T>>),
    #[serde(rename = "REJECT")]
    Reject(Response),
}

#[derive(Deserialize, Debug)]
pub struct TypedReply<'a, T> {
    #[serde(flatten)]
    data: T,
    #[serde(rename = "type")]
    type_: &'a str,
}

impl<'a, T> TryFrom<TypedReply<'a, T>> for Reply<T>
where
    T: ReplyType,
{
    type Error = IndyError;
    fn try_from(value: TypedReply<'a, T>) -> Result<Self, Self::Error> {
        if value.type_ != T::get_type() {
            Err(err_msg(IndyErrorKind::InvalidTransaction, "Invalid response type"))
        } else {
            Ok(Reply::ReplyV0(ReplyV0 { result: value.data }))
        }
    }
}

impl<'a, T> TryFrom<MessageWithTypedReply<'a, T>> for Message<T>
where
    T: ReplyType,
{
    type Error = IndyError;
    fn try_from(value: MessageWithTypedReply<'a, T>) -> Result<Self, Self::Error> {
        match value {
            MessageWithTypedReply::ReqNACK(r) => Ok(Message::ReqNACK(r)),
            MessageWithTypedReply::Reply(r) => Ok(Message::Reply(r.result().try_into()?)),
            MessageWithTypedReply::Reject(r) => Ok(Message::Reject(r)),
        }
    }
}
