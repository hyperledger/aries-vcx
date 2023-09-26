use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::ack::AckCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct Complete<T: IssuerCredentialIssuanceFormat> {
    pub ack: Option<AckCredentialV2>,
    pub _marker: PhantomData<T>,
}
