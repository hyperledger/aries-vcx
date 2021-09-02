pub const AGENCY_CONFIGS_UPDATED: &str = r#"
{
	"@type": "did:sov:123456789abcdefghi1234;spec/configs/1.0/CONFIGS_UPDATED"
}"#;
pub const DELETE_CONNECTION_DECRYPTED_RESPONSE: &str = r#"{"@type":"did:sov:123456789abcdefghi1234;spec/pairwise/1.0/CONN_STATUS_UPDATED","statusCode":"CS-103"}"#;
pub const AGENCY_MSG_STATUS_UPDATED_BY_CONNS: &str = r#"
{
    "@type": "did:sov:123456789abcdefghi1234;spec/pairwise/1.0/MSG_STATUS_UPDATED_BY_CONNS",
    "failed": [],
    "updatedUidsByConns": [
        {
            "pairwiseDID": "6FRuB95abcmzz1nURoHyWE",
            "uids": [
                "Br4CoNP4TU"
            ]
        }
    ]
}"#;