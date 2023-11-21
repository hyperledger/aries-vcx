pub const ARIES_OOB_MESSAGE: &str = r#"
{
  "@id": "testid",
  "label": "test",
  "goal_code": "p2p-messaging",
  "goal": "test",
  "services": [
    {
      "id": "did:example:123456789abcdefghi;indy",
      "type": "IndyAgent",
      "priority": 0,
      "recipientKeys": [],
      "routingKeys": [],
      "serviceEndpoint": ""
    }
  ],
  "requests~attach": []
}
"#;
