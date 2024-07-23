use crate::error::LedgerResponseParserError;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    #[allow(unused)] // unused, but part of entity
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
            //  SAFETY: Empty array cannot be instantiated
            Reply::ReplyV1(reply) => reply.data.result.into_iter().next().unwrap().result,
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
    pub result: [ReplyV0<T>; 1],
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
#[serde(try_from = "MessageWithTypedReply<'de, T>")]
#[serde(bound(deserialize = "
    Self: TryFrom<MessageWithTypedReply<'de, T>>,
    T: serde::Deserialize<'de>,
    <Self as TryFrom<MessageWithTypedReply<'de, T>>>::Error: std::fmt::Display
"))]
pub enum Message<T> {
    ReqNACK(Response),
    Reply(Reply<T>),
    Reject(Response),
}

pub trait ReplyType {
    fn get_type<'a>() -> &'a str;
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
enum MessageWithTypedReply<'a, T> {
    #[serde(rename = "REQNACK")]
    ReqNACK(Response),
    #[serde(borrow)]
    #[serde(rename = "REPLY")]
    Reply(Reply<TypedReply<'a, T>>),
    #[serde(rename = "REJECT")]
    Reject(Response),
}

#[derive(Deserialize, Debug)]
struct TypedReply<'a, T> {
    #[serde(flatten)]
    reply: T,
    #[serde(rename = "type")]
    type_: &'a str,
}

impl<'a, T> TryFrom<ReplyV0<TypedReply<'a, T>>> for ReplyV0<T>
where
    T: ReplyType,
{
    type Error = LedgerResponseParserError;

    fn try_from(value: ReplyV0<TypedReply<'a, T>>) -> Result<Self, Self::Error> {
        let expected_type = T::get_type();
        let actual_type = value.result.type_;
        if expected_type != actual_type {
            Err(LedgerResponseParserError::InvalidTransaction(format!(
                "Unexpected response type:\nExpected: {}\nActual: {}",
                expected_type, actual_type
            )))
        } else {
            Ok(ReplyV0 {
                result: value.result.reply,
            })
        }
    }
}

impl<'a, T> TryFrom<ReplyV1<TypedReply<'a, T>>> for ReplyV1<T>
where
    T: ReplyType,
{
    type Error = LedgerResponseParserError;

    fn try_from(value: ReplyV1<TypedReply<'a, T>>) -> Result<Self, Self::Error> {
        let value = value.data.result.into_iter().next().ok_or_else(|| {
            LedgerResponseParserError::InvalidTransaction("Result field is empty".to_string())
        })?;
        let data = ReplyDataV1 {
            result: [value.try_into()?],
        };
        Ok(ReplyV1 { data })
    }
}

impl<'a, T> TryFrom<Reply<TypedReply<'a, T>>> for Reply<T>
where
    T: ReplyType,
{
    type Error = LedgerResponseParserError;

    fn try_from(value: Reply<TypedReply<'a, T>>) -> Result<Self, Self::Error> {
        let reply = match value {
            Reply::ReplyV0(r) => Reply::ReplyV0(r.try_into()?),
            Reply::ReplyV1(r) => Reply::ReplyV1(r.try_into()?),
        };
        Ok(reply)
    }
}

impl<'a, T> TryFrom<MessageWithTypedReply<'a, T>> for Message<T>
where
    T: ReplyType,
{
    type Error = LedgerResponseParserError;

    fn try_from(value: MessageWithTypedReply<'a, T>) -> Result<Self, Self::Error> {
        match value {
            MessageWithTypedReply::ReqNACK(r) => Ok(Message::ReqNACK(r)),
            MessageWithTypedReply::Reply(r) => Ok(Message::Reply(r.try_into()?)),
            MessageWithTypedReply::Reject(r) => Ok(Message::Reject(r)),
        }
    }
}
