#[macro_use]
extern crate log;
extern crate serde_json;
extern crate aries_vcx_agent;
extern crate aries_vcx;


#[cfg(test)]
#[cfg(feature = "agency_pool_tests")]
mod integration_tests {
    use aries_vcx;
    use aries_vcx::agency_client::MessageStatusCode;
    use aries_vcx::handlers::connection::mediated_connection::ConnectionState;
    use aries_vcx_agent::{AgencyInitConfig, Agent as AriesAgent, InitConfig, PoolInitConfig, WalletInitConfig};
    use aries_vcx::indy::ledger::pool::test_utils::{create_tmp_genesis_txn_file};
    use aries_vcx::protocols::connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::connection::inviter::state_machine::InviterState;
    use aries_vcx::utils::devsetup::init_test_logging;

    pub async fn setup_pool() -> String {
        let genesis_path = create_tmp_genesis_txn_file();
        return genesis_path
    }

    pub async fn initialize() -> AriesAgent {
        let agent_id = uuid::Uuid::new_v4();
        let enterprise_seed= "000000000000000000000000Trustee1".to_string();
        let genesis_path = setup_pool().await;
        warn!("Genesis path: {}", genesis_path);
        let agency_endpoint = std::env::var("CLOUD_AGENCY_URL").unwrap_or("http://localhost:8080".to_string());
        let service_endpoint = format!("http://localhost:8081/didcomm");
        let init_config = InitConfig {
            enterprise_seed,
            pool_config: PoolInitConfig {
                genesis_path,
                pool_name: format!("pool_{}", agent_id),
            },
            wallet_config: WalletInitConfig {
                wallet_name: format!("rust_agent_{}", uuid::Uuid::new_v4()),
                wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".to_string(),
                wallet_kdf: "RAW".to_string(),
            },
            agency_config: Some(AgencyInitConfig {
                agency_endpoint,
                agency_did: "VsKV7grR1BUE29mG2Fm2kX".to_string(),
                agency_verkey: "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR".to_string()
            }),
            service_endpoint
        };
        AriesAgent::initialize(init_config).await.unwrap()
    }

    pub async fn connect_agents_with_mediation(
        alice: &mut AriesAgent,
        institution: &mut AriesAgent,
    ) -> String {

        let invitation = institution.mediated_connections().unwrap().create_invitation().await.unwrap();

        let thread_id = alice.mediated_connections().unwrap().receive_invitation(invitation).await.unwrap();
        alice.mediated_connections().unwrap().send_request(&thread_id).await.unwrap();
        institution.mediated_connections().unwrap().update_state(&thread_id).await.unwrap();
        alice.mediated_connections().unwrap().update_state(&thread_id).await.unwrap();
        institution.mediated_connections().unwrap().update_state(&thread_id).await.unwrap();
        assert_eq!(alice.mediated_connections().unwrap().get_state(&thread_id).unwrap(), ConnectionState::Invitee(InviteeState::Completed));
        assert_eq!(institution.mediated_connections().unwrap().get_state(&thread_id).unwrap(), ConnectionState::Inviter(InviterState::Completed));
        thread_id
    }

    #[tokio::test]
    async fn new_test_establish_connection() {
        init_test_logging();
        let mut alice = initialize().await;
        let mut faber = initialize().await;

        let thread_id = connect_agents_with_mediation(&mut alice, &mut faber).await;
        faber.mediated_connections().unwrap().send_message(&thread_id, "Hello Alice, Faber here").await.unwrap();
        let consumer_msgs = alice.mediated_connections().unwrap()
            .download_messages(&thread_id, Some(vec![MessageStatusCode::Received]), None)
            .await
            .unwrap();
        assert_eq!(consumer_msgs.len(), 1);
        assert!(consumer_msgs[0].decrypted_msg.contains("Hello Alice, Faber here"));
    }
}
