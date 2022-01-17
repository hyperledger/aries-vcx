
#[cfg(feature = "cheqd")]
#[cfg(test)]
mod domain_tests {
    use super::super::tx::{ Single, Sum, GetTxRequest,
                            ModeInfo, SignerInfo, Fee,
                            AuthInfo, TxBody, Message,
                            Tx, Any};
    use super::super::base::abci::{ Attribute, AbciMessageLog, StringEvent, TxResponse};
    use super::super::bank::{ Coin, MsgSend};
    use super::super::crypto::PubKey;
    use super::super::crypto::secp256k1::PubKey as SecpPubKey;
    use super::super::CheqdProtoBase;

    use rstest::*;
    use super::super::cheqd::v1::messages::{MsgCreateDidPayload, MsgUpdateDidPayload};
    use super::super::cheqd::v1::models::{VerificationMethod, Service};
    use std::collections::HashMap;

    /// Fixtures

    #[fixture]
    fn single() -> Single {
        Single::new(42)
    }

    #[fixture]
    fn sum(single: Single) -> Sum {
        Sum::Single(single)
    }

    #[fixture]
    fn get_tx_request() -> GetTxRequest {
        GetTxRequest::new("456".to_string())
    }

    #[fixture]
    fn mode_info(sum: Sum) -> ModeInfo {
        ModeInfo::new(Some(sum))
    }

    #[fixture]
    fn secp256k1_pub_key() -> SecpPubKey{
        SecpPubKey::new(vec![2, 59, 126, 95, 52, 102, 213, 99, 251, 102, 62, 148, 101, 72, 226, 188, 243, 222, 31, 35, 148, 19, 127, 79, 75, 79, 37, 160, 132, 193, 33, 148, 7])
    }

    #[fixture]
    fn pub_key(secp256k1_pub_key: SecpPubKey) -> PubKey {
        PubKey::Secp256k1(secp256k1_pub_key)
    }

    #[fixture]
    fn sequence() -> u64 {
        42
    }

    #[fixture]
    fn signer_info(pub_key: PubKey, mode_info: ModeInfo, sequence: u64) -> SignerInfo {
        SignerInfo::new(Some(pub_key), Some(mode_info), sequence)
    }

    #[fixture]
    fn coin() -> Coin {
        Coin {
            denom: "ncheq".to_string(),
            amount: "100500".to_string(),
        }
    }

    #[fixture]
    fn fee(coin: Coin) -> Fee {
        Fee {
            amount: vec![coin],
            gas_limit: 0,
            payer: "".to_string(),
            granter: "".to_string()
        }
    }

    #[fixture]
    fn auth_info(signer_info: SignerInfo, fee: Fee) -> AuthInfo {
        AuthInfo {
            signer_infos: vec![signer_info],
            fee: Some(fee)
        }
    }

    #[fixture]
    fn msg_create_did() -> MsgCreateDidPayload {
        let verification_method = VerificationMethod::new(
            "id".into(),
            "type".into(),
            "controller".into(),
            HashMap::new(),
            "public_key_multibase".into()
        );

        let did_service = Service::new(
            "id".into(),
            "type".into(),
            "service_endpoint".into()
        );

        MsgCreateDidPayload::new(
            vec!("context".to_string()),
            "id".into(),
            vec!("controller".to_string()),
            vec!(verification_method),
            vec!("authentication".to_string()),
            vec!("assertion_method".to_string()),
            vec!("capability_invocation".to_string()),
            vec!("capability_delegation".to_string()),
            vec!("key_agreement".to_string()),
            vec!(did_service),
            vec!("also_known_as".to_string()),
        )
    }

    #[fixture]
    fn msg_update_did() -> MsgUpdateDidPayload {
        let verification_method = VerificationMethod::new(
            "id".into(),
            "type".into(),
            "controller".into(),
            HashMap::new(),
            "public_key_multibase".into()
        );

        let did_service = Service::new(
            "id".into(),
            "type".into(),
            "service_endpoint".into()
        );

        MsgUpdateDidPayload::new(
            vec!("context".to_string()),
            "id".into(),
            vec!("controller".to_string()),
            vec!(verification_method),
            vec!("authentication".to_string()),
            vec!("assertion_method".to_string()),
            vec!("capability_invocation".to_string()),
            vec!("capability_delegation".to_string()),
            vec!("key_agreement".to_string()),
            vec!(did_service),
            vec!("also_known_as".to_string()),
            "version_id".to_string(),
        )
    }

    #[fixture]
    fn msg_send(coin: Coin) -> MsgSend {
        MsgSend {
            from_address: "From".to_string(),
            to_address: "To".to_string(),
            amount: vec![coin]
        }
    }
    #[fixture]
    fn any() -> Any {
        Any {
            type_url: "any_type".to_string(),
            value: vec![1,2,3,4,5,]
        }
    }

    #[fixture]
    fn message(msg_send: MsgSend) -> Message {
        Message::MsgSend(msg_send)
    }

    #[fixture]
    fn tx_body(message: Message, any: Any) -> TxBody {
        TxBody {
            messages: vec![message],
            memo: "".to_string(),
            timeout_height: 0,
            extension_options: vec![any.clone()],
            non_critical_extension_options: vec![any.clone()]
        }
    }

    #[fixture]
    fn signature() -> Vec<u8> {
        vec![132, 232, 65, 244, 3, 108, 251, 129, 34, 75, 181, 126, 95, 189, 80, 244, 161, 179, 18, 17, 12, 181, 101, 42, 46, 29, 188, 168, 70, 159, 163, 223, 117, 146, 162, 229, 80, 83, 80, 24, 204, 91, 180, 65, 191, 173, 161, 253, 139, 208, 50, 36, 197, 75, 63, 241, 58, 228, 46, 108, 87, 204, 14, 248]
    }

    #[fixture]
    fn tx(tx_body: TxBody, auth_info: AuthInfo, signature: Vec<u8>) -> Tx{
        Tx {
            body: Some(tx_body),
            auth_info: Some(auth_info),
            signatures: vec![signature]
        }
    }

    #[fixture]
    fn attribute() -> Attribute {
        Attribute {
            key: "action".to_string(),
            value: "CreateNYM".to_string()
        }
    }

    #[fixture]
    fn string_event(attribute: Attribute) -> StringEvent {
        StringEvent {
            r#type: "message".to_string(),
            attributes: vec![attribute]
        }
    }

    #[fixture]
    fn abci_message_log(string_event: StringEvent) -> AbciMessageLog {
        AbciMessageLog {
            msg_index: 0,
            log: "".to_string(),
            events: vec![string_event]
        }
    }

    #[fixture]
    fn tx_response(abci_message_log: AbciMessageLog, any: Any) -> TxResponse {
        TxResponse {
            height: 6594,
            txhash: "69B4B8F4BA1D62D82D56AF5CF487D1388FA1E4E3617BD6B3083D65FD3ACE800B".to_string(),
            codespace: "".to_string(),
            code: 0,
            data: "0A0F0A094372656174654E796D12020836".to_string(),
            raw_log: "[{\"events \": [{\"type\": \"message\",\"attributes \": [{\"key \": \"action \",\"value\": \"CreateNym \"}]}] }],".to_string(),
            logs: vec![abci_message_log],
            info: "".to_string(),
            gas_wanted: 300000,
            gas_used: 46507,
            tx: Some(any),
            timestamp: "2021-09-15T07:40:01Z".to_string()
        }
    }

    /// Tests

    #[rstest]
    fn test_single(single: Single) {

        let proto = single.to_proto().unwrap();
        let decoded = Single::from_proto(&proto).unwrap();

        assert_eq!(single, decoded);
    }

    #[rstest]
    fn test_get_tx_request(get_tx_request: GetTxRequest) {

        let proto = get_tx_request.to_proto().unwrap();
        let decoded = GetTxRequest::from_proto(&proto).unwrap();

        assert_eq!(get_tx_request, decoded);
    }

    #[rstest]
    fn test_mode_info(mode_info: ModeInfo) {

        let proto = mode_info.to_proto().unwrap();
        let decoded = ModeInfo::from_proto(&proto).unwrap();

        assert_eq!(mode_info, decoded);
    }

    #[rstest]
    fn test_pubkey(pub_key: PubKey) {
        let proto = pub_key.to_proto().unwrap();
        let decoded = PubKey::from_proto(&proto).unwrap();

        assert_eq!(pub_key, decoded);
    }

    #[rstest]
    fn test_signer_info(signer_info: SignerInfo) {
        let proto = signer_info.to_proto().unwrap();
        let decoded = SignerInfo::from_proto(&proto).unwrap();

        assert_eq!(signer_info, decoded);
    }

    #[rstest]
    fn test_coin(coin: Coin) {
        let proto = coin.to_proto().unwrap();
        let decoded = Coin::from_proto(&proto).unwrap();

        assert_eq!(coin, decoded);
    }

    #[rstest]
    fn test_fee(fee: Fee) {
        let proto = fee.to_proto().unwrap();
        let decoded = Fee::from_proto(&proto).unwrap();

        assert_eq!(fee, decoded);
    }

    #[rstest]
    fn test_auth_info(auth_info: AuthInfo) {
        let proto = auth_info.to_proto().unwrap();
        let decoded = AuthInfo::from_proto(&proto).unwrap();

        assert_eq!(auth_info, decoded);
    }

    #[rstest]
    fn test_tx_body(tx_body: TxBody) {
        let proto = tx_body.to_proto().unwrap();
        let decoded = TxBody::from_proto(&proto).unwrap();

        assert_eq!(tx_body, decoded);
    }

    #[rstest]
    fn test_tx(tx: Tx) {
        let proto = tx.to_proto().unwrap();
        let decoded = Tx::from_proto(&proto).unwrap();

        assert_eq!(tx, decoded);
    }

    #[rstest]
    fn test_abci_message_log(abci_message_log: AbciMessageLog) {
        let proto = abci_message_log.to_proto().unwrap();
        let decoded = AbciMessageLog::from_proto(&proto).unwrap();

        assert_eq!(abci_message_log, decoded);
    }

    #[rstest]
    fn test_string_event(string_event: StringEvent) {
        let proto = string_event.to_proto().unwrap();
        let decoded = StringEvent::from_proto(&proto).unwrap();

        assert_eq!(string_event, decoded);
    }

    #[rstest]
    fn test_attribute(attribute: Attribute) {
        let proto = attribute.to_proto().unwrap();
        let decoded = Attribute::from_proto(&proto).unwrap();

        assert_eq!(attribute, decoded);
    }

    #[rstest]
    fn test_tx_response(tx_response: TxResponse) {
        let proto = tx_response.to_proto().unwrap();
        let decoded = TxResponse::from_proto(&proto).unwrap();

        assert_eq!(tx_response, decoded);
    }

    #[rstest]
    fn test_msg_create_did(msg_create_did: MsgCreateDidPayload) {
        let proto = msg_create_did.to_proto().unwrap();
        let decoded = MsgCreateDidPayload::from_proto(&proto).unwrap();

        assert_eq!(msg_create_did, decoded);
    }

    #[rstest]
    fn test_msg_update_did(msg_update_did: MsgUpdateDidPayload) {
        let proto = msg_update_did.to_proto().unwrap();
        let decoded = MsgUpdateDidPayload::from_proto(&proto).unwrap();

        assert_eq!(msg_update_did, decoded);
    }

    #[rstest]
    fn test_msg_send(msg_send: MsgSend) {
        let proto = msg_send.to_proto().unwrap();
        let decoded = MsgSend::from_proto(&proto).unwrap();

        assert_eq!(msg_send, decoded);
    }

}
