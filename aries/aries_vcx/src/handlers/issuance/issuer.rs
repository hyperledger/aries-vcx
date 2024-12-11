use aries_vcx_anoncreds::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use messages::{
    misc::MimeType,
    msg_fields::protocols::{
        cred_issuance::{
            common::CredentialAttr,
            v1::{
                ack::AckCredentialV1, issue_credential::IssueCredentialV1,
                offer_credential::OfferCredentialV1, propose_credential::ProposeCredentialV1,
                request_credential::RequestCredentialV1, CredentialIssuanceV1, CredentialPreviewV1,
            },
            CredentialIssuance,
        },
        notification::Notification,
        report_problem::ProblemReport,
    },
    AriesMessage,
};

use crate::{
    errors::error::prelude::*,
    handlers::util::OfferInfo,
    protocols::issuance::issuer::state_machine::{IssuerSM, IssuerState, RevocationInfoV1},
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Issuer {
    issuer_sm: IssuerSM,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerConfig {
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

fn _build_credential_preview(credential_json: &str) -> VcxResult<CredentialPreviewV1> {
    trace!(
        "Issuer::_build_credential_preview >>> credential_json: {:?}",
        secret!(credential_json)
    );
    let cred_values: serde_json::Value = serde_json::from_str(credential_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Can't deserialize credential preview json. credential_json: {}, error: {:?}",
                credential_json, err
            ),
        )
    })?;

    // todo: should throw err if cred_values is not serde_json::Value::Array or
    // serde_json::Value::Object
    let mut attributes = Vec::new();

    match cred_values {
        serde_json::Value::Array(cred_values) => {
            for cred_value in cred_values.iter() {
                let key = cred_value.get("name").ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("No 'name' field in cred_value: {:?}", cred_value),
                ))?;
                let value = cred_value.get("value").ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("No 'value' field in cred_value: {:?}", cred_value),
                ))?;

                let name = key
                    .as_str()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidOption,
                        "Credential value names are currently only allowed to be strings",
                    ))?
                    .to_owned();

                let value = value
                    .as_str()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidOption,
                        "Credential values are currently only allowed to be strings",
                    ))?
                    .to_owned();

                let attr = CredentialAttr::builder()
                    .name(name)
                    .value(value)
                    .mime_type(MimeType::Plain)
                    .build();

                attributes.push(attr);
            }
        }
        serde_json::Value::Object(values_map) => {
            for item in values_map.iter() {
                let (key, value) = item;
                let value = value
                    .as_str()
                    .ok_or(AriesVcxError::from_msg(
                        AriesVcxErrorKind::InvalidOption,
                        "Credential values are currently only allowed to be strings",
                    ))?
                    .to_owned();
                let attr = CredentialAttr::builder()
                    .name(key.to_owned())
                    .value(value)
                    .mime_type(MimeType::Plain)
                    .build();

                attributes.push(attr);
            }
        }
        _ => {}
    };

    Ok(CredentialPreviewV1::new(attributes))
}

impl Issuer {
    pub fn create(source_id: &str) -> VcxResult<Issuer> {
        trace!("Issuer::create >>> source_id: {:?}", source_id);
        let issuer_sm = IssuerSM::new(source_id);
        Ok(Issuer { issuer_sm })
    }

    pub fn create_from_proposal(
        source_id: &str,
        credential_proposal: &ProposeCredentialV1,
    ) -> VcxResult<Issuer> {
        trace!(
            "Issuer::create_from_proposal >>> source_id: {:?}, credential_proposal: {:?}",
            source_id,
            credential_proposal
        );
        let issuer_sm = IssuerSM::from_proposal(source_id, credential_proposal);
        Ok(Issuer { issuer_sm })
    }

    // todo: "build_credential_offer_msg" should take optional revReg as parameter, build OfferInfo
    // from that
    pub async fn build_credential_offer_msg(
        &mut self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
        offer_info: OfferInfo,
        comment: Option<String>,
    ) -> VcxResult<()> {
        let credential_preview = _build_credential_preview(&offer_info.credential_json)?;
        let libindy_cred_offer = anoncreds
            .issuer_create_credential_offer(wallet, &offer_info.cred_def_id)
            .await?;
        self.issuer_sm = self.issuer_sm.clone().build_credential_offer_msg(
            &serde_json::to_string(&libindy_cred_offer)?,
            credential_preview,
            comment,
            &offer_info,
        )?;
        Ok(())
    }

    pub fn get_credential_offer(&self) -> VcxResult<OfferCredentialV1> {
        self.issuer_sm.get_credential_offer_msg()
    }

    pub fn get_credential_offer_msg(&self) -> VcxResult<AriesMessage> {
        let offer = self.issuer_sm.get_credential_offer_msg()?;
        Ok(offer.into())
    }

    pub fn process_credential_request(&mut self, request: RequestCredentialV1) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_request(request)?;
        Ok(())
    }

    pub fn process_credential_ack(&mut self, ack: AckCredentialV1) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_ack(ack)?;
        Ok(())
    }

    pub async fn build_credential(
        &mut self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
    ) -> VcxResult<()> {
        self.issuer_sm = self
            .issuer_sm
            .clone()
            .build_credential(wallet, anoncreds)
            .await?;
        Ok(())
    }

    pub fn get_msg_issue_credential(&mut self) -> VcxResult<IssueCredentialV1> {
        self.issuer_sm.clone().get_msg_issue_credential()
    }

    pub fn get_state(&self) -> IssuerState {
        self.issuer_sm.get_state()
    }

    pub fn get_source_id(&self) -> VcxResult<String> {
        Ok(self.issuer_sm.get_source_id())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.issuer_sm.is_terminal_state()
    }

    pub fn get_revocation_id(&self) -> VcxResult<u32> {
        self.issuer_sm
            .get_revocation_info()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Credential has not yet been created",
            ))?
            .cred_rev_id
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Credential has not yet been created or is irrevocable",
            ))
            .and_then(|s| s.parse().map_err(Into::into))
    }

    pub async fn revoke_credential_local(
        &self,
        wallet: &impl BaseWallet,
        anoncreds: &impl BaseAnonCreds,
        ledger: &impl AnoncredsLedgerRead,
    ) -> VcxResult<()> {
        let revocation_info: RevocationInfoV1 =
            self.issuer_sm
                .get_revocation_info()
                .ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidState,
                    "Credential is not revocable, no revocation info has been found.",
                ))?;
        if let (Some(cred_rev_id), Some(rev_reg_id), Some(_tails_file)) = (
            revocation_info.cred_rev_id,
            revocation_info.rev_reg_id,
            revocation_info.tails_file,
        ) {
            #[allow(deprecated)] // TODO - https://github.com/hyperledger/aries-vcx/issues/1309
            let rev_reg_delta_json = ledger
                .get_rev_reg_delta_json(&rev_reg_id.to_owned().try_into()?, None, None)
                .await?
                .0;
            anoncreds
                .revoke_credential_local(
                    wallet,
                    &rev_reg_id.try_into()?,
                    cred_rev_id.parse()?,
                    rev_reg_delta_json,
                )
                .await?;
        } else {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Revocation info is not complete, cannot revoke credential.",
            ));
        }
        Ok(())
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_rev_reg_id()
    }

    pub fn get_rev_id(&self) -> VcxResult<u32> {
        self.issuer_sm.get_rev_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.issuer_sm.thread_id()
    }

    pub fn get_proposal(&self) -> VcxResult<ProposeCredentialV1> {
        self.issuer_sm.get_proposal()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.credential_status())
    }

    pub fn is_revokable(&self) -> bool {
        self.issuer_sm.is_revokable()
    }

    pub async fn is_revoked(&self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<bool> {
        self.issuer_sm.is_revoked(ledger).await
    }

    pub async fn receive_proposal(&mut self, proposal: ProposeCredentialV1) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_proposal(proposal)?;
        Ok(())
    }

    pub async fn receive_request(&mut self, request: RequestCredentialV1) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_request(request)?;
        Ok(())
    }

    pub async fn receive_ack(&mut self, ack: AckCredentialV1) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_ack(ack)?;
        Ok(())
    }

    pub async fn receive_problem_report(&mut self, problem_report: ProblemReport) -> VcxResult<()> {
        self.issuer_sm = self
            .issuer_sm
            .clone()
            .receive_problem_report(problem_report)?;
        Ok(())
    }

    pub fn get_problem_report(&self) -> VcxResult<ProblemReport> {
        self.issuer_sm.get_problem_report()
    }

    // todo: will ultimately end up in generic SM layer
    pub async fn process_aries_msg(&mut self, msg: AriesMessage) -> VcxResult<()> {
        let issuer_sm = match msg {
            AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::ProposeCredential(proposal),
            )) => self.issuer_sm.clone().receive_proposal(proposal)?,
            AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::RequestCredential(request),
            )) => self.issuer_sm.clone().receive_request(request)?,
            AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::Ack(ack),
            )) => self.issuer_sm.clone().receive_ack(ack)?,
            AriesMessage::ReportProblem(report) => {
                self.issuer_sm.clone().receive_problem_report(report)?
            }
            AriesMessage::Notification(Notification::ProblemReport(report)) => self
                .issuer_sm
                .clone()
                .receive_problem_report(report.into())?,
            AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::ProblemReport(report),
            )) => self
                .issuer_sm
                .clone()
                .receive_problem_report(report.into())?,
            _ => self.issuer_sm.clone(),
        };
        self.issuer_sm = issuer_sm;
        Ok(())
    }
}
