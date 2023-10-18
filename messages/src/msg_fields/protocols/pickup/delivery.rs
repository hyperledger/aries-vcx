use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use typed_builder::TypedBuilder;

use super::decorators::PickupDecoratorsCommon;
use crate::msg_parts::MsgParts;

pub type Delivery = MsgParts<DeliveryContent, PickupDecoratorsCommon>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct DeliveryContent {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient_key: Option<String>,
    #[serde(rename = "~attach")]
    pub attach: Vec<DeliveryAttach>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct DeliveryAttach {
    #[serde(rename = "@id")]
    pub id: String,
    pub data: DeliveryAttachData,
}

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct DeliveryAttachData {
    #[serde_as(as = "Base64")]
    pub base64: Vec<u8>,
}
