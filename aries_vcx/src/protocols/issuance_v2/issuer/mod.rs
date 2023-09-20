use messages::msg_fields::protocols::{cred_issuance::CredentialPreview, notification::ack::Ack};

use crate::{
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    protocols::issuance_v2::{messages::OfferCredentialV2, RecoveredSMError},
};

use self::states::{Complete, CredentialPrepared, OfferPrepared, ProposalReceived, RequestReceived};

use super::{
    formats::issuer::IssuerCredentialIssuanceFormat,
    messages::{IssueCredentialV2, RequestCredentialV2},
    VcxSMTransitionResult,
};

pub mod states {
    use messages::msg_fields::protocols::notification::ack::Ack;

    use crate::protocols::issuance_v2::messages::{
        IssueCredentialV2, OfferCredentialV2, ProposeCredentialV2, RequestCredentialV2,
    };

    pub struct ProposalReceived {
        pub proposal: ProposeCredentialV2,
    }

    pub struct OfferPrepared {
        pub offer: OfferCredentialV2,
        pub credentials_remaining: u32,
    }

    pub struct RequestReceived {
        pub offer: Option<OfferCredentialV2>,
        pub credentials_remaining: Option<u32>,
        pub request: RequestCredentialV2,
    }

    pub struct CredentialPrepared {
        pub credential: IssueCredentialV2,
        pub credentials_remaining: u32,
    }

    pub struct Complete {
        pub ack: Option<Ack>,
    }
}

pub struct IssuerV2<S> {
    state: S,
    thread_id: String,
}

impl IssuerV2<ProposalReceived> {
    pub fn receive_proposal(proposal: ProposalReceived) -> Self {
        _ = proposal;
        todo!()
    }

    pub async fn prepare_offer<T: IssuerCredentialIssuanceFormat>(
        self,
        input_data: &T::CreateOfferInput,
        number_of_credentials_available: Option<u32>, // defaults to 1 if None
        preview: Option<CredentialPreview>, // TODO - is this the right format? may not be versioned correctly...
    ) -> VcxSMTransitionResult<IssuerV2<OfferPrepared>, Self> {
        let attachment_data = match T::create_offer_attachment_content(input_data).await {
            Ok(data) => data,
            Err(error) => {
                return Err(RecoveredSMError {
                    error,
                    state_machine: self,
                })
            }
        };

        let _offer_attachment_format = T::get_offer_attachment_format();
        // create offer msg with the attachment data and format
        _ = attachment_data;
        _ = preview;
        let multi_available = number_of_credentials_available.unwrap_or(1);
        _ = multi_available;
        let offer = OfferCredentialV2;

        let new_state = OfferPrepared {
            offer,
            credentials_remaining: multi_available,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: String::new(),
        })
    }

    // TODO - helpers so that consumers can understand what proposal they received?
}

impl IssuerV2<OfferPrepared> {
    pub async fn with_offer<T: IssuerCredentialIssuanceFormat>(
        input_data: &T::CreateOfferInput,
        number_of_credentials_available: Option<u32>, // defaults to 1 if None
        preview: Option<CredentialPreview>, // TODO - is this the right format? may not be versioned correctly...
    ) -> VcxResult<Self> {
        let attachment_data = T::create_offer_attachment_content(input_data).await?;

        let _offer_attachment_format = T::get_offer_attachment_format();
        // create offer msg with the attachment data and format
        _ = attachment_data;
        _ = preview;
        let multi_available = number_of_credentials_available.unwrap_or(1);
        _ = multi_available;
        let offer = OfferCredentialV2;

        let new_state = OfferPrepared {
            offer,
            credentials_remaining: multi_available,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: String::new(),
        })
    }

    pub fn get_offer(&self) -> &OfferCredentialV2 {
        &self.state.offer
    }

    pub fn receive_request(self, request: RequestCredentialV2) -> IssuerV2<RequestReceived> {
        let new_state = RequestReceived {
            offer: Some(self.state.offer),
            credentials_remaining: Some(self.state.credentials_remaining),
            request,
        };

        IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }
}

impl IssuerV2<RequestReceived> {
    pub fn with_request<T: IssuerCredentialIssuanceFormat>(request: RequestCredentialV2) -> VcxResult<Self> {
        if !T::supports_request_independent_of_offer() {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::ActionNotSupported,
                "Receiving a request independent of an offer is unsupported for this format",
            ));
        }

        let new_state = RequestReceived {
            offer: None,
            request,
            credentials_remaining: None,
        };

        Ok(Self {
            state: new_state,
            thread_id: String::new(), // request.id/thid
        })
    }

    pub async fn prepare_credential<T: IssuerCredentialIssuanceFormat>(
        self,
        input_data: &T::CreateCredentialInput,
        more_credentials_available: Option<u32>, // defaults to the current state's (`credentials_remaining` - 1), else 0
    ) -> VcxSMTransitionResult<IssuerV2<CredentialPrepared>, Self> {
        let request = &self.state.request;

        let res = match &self.state.offer {
            Some(offer) => T::create_credential_attachment_content(offer, request, input_data).await,
            None => T::create_credential_attachment_content_independent_of_offer(request, input_data).await,
        };

        let attachment_data = match res {
            Ok(data) => data,
            Err(error) => {
                return Err(RecoveredSMError {
                    error,
                    state_machine: self,
                })
            }
        };

        let _credential_attachment_format = T::get_credential_attachment_format();
        // create cred msg with the attachment data and format
        _ = attachment_data;
        let more_available = more_credentials_available
            .or(self.state.credentials_remaining.map(|x| x - 1))
            .unwrap_or(0);
        let credential = IssueCredentialV2;

        let new_state = CredentialPrepared {
            credential,
            credentials_remaining: more_available,
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: String::new(),
        })
    }
}

impl IssuerV2<CredentialPrepared> {
    pub fn get_credential(&self) -> &IssueCredentialV2 {
        &self.state.credential
    }

    pub fn remaining_credentials_to_issue(&self) -> u32 {
        self.state.credentials_remaining
    }

    pub fn receive_request_for_more(
        self,
        request: RequestCredentialV2,
    ) -> VcxSMTransitionResult<IssuerV2<RequestReceived>, Self> {
        if self.remaining_credentials_to_issue() == 0 {
            return Err(RecoveredSMError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "Received a request when all intended credentials have already been issued",
                ),
                state_machine: self,
            });
        }

        let new_state = RequestReceived {
            offer: None,
            request,
            credentials_remaining: Some(764),
        };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    pub fn is_expecting_ack(&self) -> bool {
        // read issue cred payload
        true
    }

    pub fn complete_without_ack(self) -> VcxSMTransitionResult<IssuerV2<Complete>, Self> {
        if self.is_expecting_ack() {
            return Err(RecoveredSMError {
                error: AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "Cannot transition until ACK is received",
                ),
                state_machine: self,
            });
        }

        let new_state = Complete { ack: None };

        Ok(IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        })
    }

    pub fn complete_with_ack(self, ack: Ack) -> IssuerV2<Complete> {
        let new_state = Complete { ack: Some(ack) };

        IssuerV2 {
            state: new_state,
            thread_id: self.thread_id,
        }
    }
}
