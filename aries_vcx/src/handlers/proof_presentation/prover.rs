use std::collections::HashMap;

use indy_sys::{WalletHandle, PoolHandle};

use agency_client::agency_client::AgencyClient;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::libindy::utils::anoncreds;
use crate::messages::a2a::A2AMessage;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_proposal::{PresentationPreview, PresentationProposalData};
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::protocols::proof_presentation::prover::messages::ProverMessages;
use crate::protocols::proof_presentation::prover::state_machine::{ProverSM, ProverState};
use crate::protocols::SendClosure;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Prover {
    prover_sm: ProverSM,
}

impl Prover {
    pub fn create(source_id: &str) -> VcxResult<Prover> {
        trace!("Prover::create >>> source_id: {}", source_id);
        Ok(Prover {
            prover_sm: ProverSM::new(source_id.to_string()),
        })
    }

    pub fn create_from_request(source_id: &str, presentation_request: PresentationRequest) -> VcxResult<Prover> {
        trace!(
            "Prover::create_from_request >>> source_id: {}, presentation_request: {:?}",
            source_id,
            presentation_request
        );
        Ok(Prover {
            prover_sm: ProverSM::from_request(presentation_request, source_id.to_string()),
        })
    }

    pub fn get_state(&self) -> ProverState {
        self.prover_sm.get_state()
    }

    pub fn presentation_status(&self) -> u32 {
        trace!("Prover::presentation_state >>>");
        self.prover_sm.presentation_status()
    }

    pub async fn retrieve_credentials(&self, wallet_handle: WalletHandle) -> VcxResult<String> {
        trace!("Prover::retrieve_credentials >>>");
        let presentation_request = self.presentation_request_data()?;
        anoncreds::libindy_prover_get_credentials_for_proof_req(wallet_handle, &presentation_request).await
    }

    pub async fn generate_presentation(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        credentials: String,
        self_attested_attrs: String,
    ) -> VcxResult<()> {
        trace!(
            "Prover::generate_presentation >>> credentials: {}, self_attested_attrs: {:?}",
            credentials,
            self_attested_attrs
        );
        self.step(
            wallet_handle,
            pool_handle,
            ProverMessages::PreparePresentation((credentials, self_attested_attrs)),
            None,
        )
        .await
    }

    pub fn generate_presentation_msg(&self) -> VcxResult<String> {
        trace!("Prover::generate_presentation_msg >>>");
        let proof = self.prover_sm.presentation()?.to_owned();
        Ok(json!(proof).to_string())
    }

    pub async fn set_presentation(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle, presentation: Presentation) -> VcxResult<()> {
        trace!("Prover::set_presentation >>>");
        self.step(wallet_handle, pool_handle, ProverMessages::SetPresentation(presentation), None)
            .await
    }

    pub async fn send_proposal(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        proposal_data: PresentationProposalData,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        trace!("Prover::send_proposal >>>");
        self.step(
            wallet_handle,
            pool_handle,
            ProverMessages::PresentationProposalSend(proposal_data),
            Some(send_message),
        )
        .await
    }

    pub async fn send_presentation(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle,send_message: SendClosure) -> VcxResult<()> {
        trace!("Prover::send_presentation >>>");
        self.step(wallet_handle, pool_handle, ProverMessages::SendPresentation, Some(send_message))
            .await
    }

    pub fn progressable_by_message(&self) -> bool {
        self.prover_sm.progressable_by_message()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.prover_sm.find_message_to_handle(messages)
    }

    pub async fn handle_message(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        message: ProverMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        trace!("Prover::handle_message >>> message: {:?}", message);
        self.step(wallet_handle, pool_handle, message, send_message).await
    }

    pub fn presentation_request_data(&self) -> VcxResult<String> {
        self.prover_sm
            .presentation_request()?
            .request_presentations_attach
            .content()
    }

    pub fn get_proof_request_attachment(&self) -> VcxResult<String> {
        let data = self
            .prover_sm
            .presentation_request()?
            .request_presentations_attach
            .content()?;
        let proof_request_data: serde_json::Value = serde_json::from_str(&data).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize {:?} into PresentationRequestData: {:?}", data, err),
            )
        })?;
        Ok(proof_request_data.to_string())
    }

    pub fn get_source_id(&self) -> String {
        self.prover_sm.source_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.prover_sm.get_thread_id()
    }

    pub async fn step(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        message: ProverMessages,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        self.prover_sm = self
            .prover_sm
            .clone()
            .step(wallet_handle, pool_handle, message, send_message)
            .await?;
        Ok(())
    }

    pub async fn decline_presentation_request(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        send_message: SendClosure,
        reason: Option<String>,
        proposal: Option<String>,
    ) -> VcxResult<()> {
        trace!(
            "Prover::decline_presentation_request >>> reason: {:?}, proposal: {:?}",
            reason,
            proposal
        );
        match (reason, proposal) {
            (Some(reason), None) => {
                self.step(
                    wallet_handle,
                    pool_handle,
                    ProverMessages::RejectPresentationRequest(reason),
                    Some(send_message),
                )
                .await
            }
            (None, Some(proposal)) => {
                let presentation_preview: PresentationPreview = serde_json::from_str(&proposal).map_err(|err| {
                    VcxError::from_msg(
                        VcxErrorKind::InvalidJson,
                        format!("Cannot serialize Presentation Preview: {:?}", err),
                    )
                })?;

                self.step(
                    wallet_handle,
                    pool_handle,
                    ProverMessages::ProposePresentation(presentation_preview),
                    Some(send_message),
                )
                .await
            }
            (None, None) => Err(VcxError::from_msg(
                VcxErrorKind::InvalidOption,
                "Either `reason` or `proposal` parameter must be specified.",
            )),
            (Some(_), Some(_)) => Err(VcxError::from_msg(
                VcxErrorKind::InvalidOption,
                "Only one of `reason` or `proposal` parameters must be specified.",
            )),
        }
    }

    pub async fn update_state(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<ProverState> {
        trace!("Prover::update_state >>> ");
        if !self.progressable_by_message() {
            return Ok(self.get_state());
        }
        let send_message = connection.send_message_closure(wallet_handle, pool_handle)?;

        let messages = connection.get_messages(pool_handle, agency_client).await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(wallet_handle, pool_handle, msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid, agency_client).await?;
        }
        Ok(self.get_state())
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use indy_sys::PoolHandle;

    use agency_client::agency_client::AgencyClient;

    use crate::error::prelude::*;
    use crate::handlers::connection::connection::Connection;
    use crate::messages::a2a::A2AMessage;

    pub async fn get_proof_request_messages(
        pool_handle: PoolHandle,
        agency_client: &AgencyClient,
        connection: &Connection,
    ) -> VcxResult<String> {
        let presentation_requests: Vec<A2AMessage> = connection
            .get_messages(pool_handle, agency_client)
            .await?
            .into_iter()
            .filter_map(|(_, message)| match message {
                A2AMessage::PresentationRequest(_) => Some(message),
                _ => None,
            })
            .collect();

        Ok(json!(presentation_requests).to_string())
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod tests {
    use crate::messages::proof_presentation::presentation_request::PresentationRequest;
    use crate::utils::devsetup::*;

    use super::*;

    #[tokio::test]
    async fn test_retrieve_credentials_fails_with_no_proof_req() {
        let setup = SetupLibraryWallet::init().await;

        let proof_req = PresentationRequest::create();
        let proof = Prover::create_from_request("1", proof_req).unwrap();
        assert_eq!(
            proof
                .retrieve_credentials(setup.wallet_handle)
                .await
                .unwrap_err()
                .kind(),
            VcxErrorKind::InvalidJson
        );
    }
}
