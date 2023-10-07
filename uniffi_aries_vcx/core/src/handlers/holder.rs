use aries_vcx::{
    handlers::issuance::holder::Holder as VcxHolder,
    messages::msg_fields::protocols::cred_issuance::v1::{
        offer_credential::OfferCredentialV1, propose_credential::ProposeCredentialV1,
    },
};
use std::sync::{Arc, Mutex};

use crate::{errors::error::VcxUniFFIResult, runtime::block_on};
pub struct Holder {
    handler: Mutex<VcxHolder>,
}

// initializers

pub fn create(source_id: String) -> VcxUniFFIResult<Arc<Holder>> {
    block_on(async {
        // FIXME: remove .unwrap()
        let handler = Mutex::new(VcxHolder::create(&source_id).unwrap());
        Ok(Arc::new(Holder { handler }))
    })
}

pub fn create_from_offer(
    source_id: String,
    cred: OfferCredentialV1,
) -> VcxUniFFIResult<Arc<Holder>> {
    block_on(async {
        // FIXME: remove .unwrap()
        let handler = Mutex::new(VcxHolder::create_from_offer(&source_id, cred).unwrap());
        Ok(Arc::new(Holder { handler }))
    })
}

pub fn create_with_proposal(
    source_id: String,
    propose_credential: ProposeCredentialV1,
) -> VcxUniFFIResult<Arc<Holder>> {
    block_on(async {
        // FIXME: remove .unwrap()
        let handler =
            Mutex::new(VcxHolder::create_with_proposal(&source_id, propose_credential).unwrap());
        Ok(Arc::new(Holder { handler }))
    })
}

impl Holder {}
