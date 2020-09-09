// Alice receives invitation via out of band channel
pub const ARIES_CONNECTION_INVITATION: &str = r#"
{
    "@id": "28b39b79-f5db-4478-879a-15bb12632d00",
    "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
    "label": "alice-e9b498a1-7d86-4389-a9de-3823dbb2f27e",
    "recipientKeys": [
        "DEKbrMDX9LBGhCk4LBhH6t5B6Kh5iE7GvfepAJYXp7GX"
    ],
    "routingKeys": [
        "C9JGq5BLcZNAQZ3x27w9cHTA7N6dysZThjLjjPRbvDoC",
        "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
    ],
    "serviceEndpoint": "http://localhost:8080/agency/msg"
}"#;

// Alice sends connection request to Faber
pub const ARIES_CONNECTION_REQUEST: &str = r#"
{
    "@id": "b5517062-303f-4267-9a29-09bc89497c06",
    "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/request",
    "connection": {
        "DID": "2RjtVytftf9Psbh3E8jqyq",
        "DIDDoc": {
            "@context": "https://w3id.org/did/v1",
            "authentication": [
                {
                    "publicKey": "2RjtVytftf9Psbh3E8jqyq#1",
                    "type": "Ed25519SignatureAuthentication2018"
                }
            ],
            "id": "2RjtVytftf9Psbh3E8jqyq",
            "publicKey": [
                {
                    "controller": "2RjtVytftf9Psbh3E8jqyq",
                    "id": "1",
                    "publicKeyBase58": "n6ZJrPGhbkLxQBxH11BvQHSKch58sx3MAqDTkUG4GmK",
                    "type": "Ed25519VerificationKey2018"
                }
            ],
            "service": [
                {
                    "id": "did:example:123456789abcdefghi;indy",
                    "priority": 0,
                    "recipientKeys": [
                        "2RjtVytftf9Psbh3E8jqyq#1"
                    ],
                    "routingKeys": [
                        "AKnC8qR9xsZZEBY7mdV6fzjmmtKxeegrNatpz4jSJhrH",
                        "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                    ],
                    "serviceEndpoint": "http://localhost:8080/agency/msg",
                    "type": "IndyAgent"
                }
            ]
        }
    },
    "label": "alice-157ea14b-4b7c-48a5-b536-d4ed6e027b84"
}"#;

// Faber sends connection response to Alice, using thid value as was @id in connection request
pub const ARIES_CONNECTION_RESPONSE: &str = r#"
{
    "@id": "586c54a1-8fcf-4539-aebe-19bf02567653",
    "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/response",
    "connection~sig": {
        "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/signature/1.0/ed25519Sha512_single",
        "sig_data": "AAAAAF9aIsl7IkRJRCI6IjNZcThnclM2eWNUemFDallBZjRQSkQiLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImF1dGhlbnRpY2F0aW9uIjpbeyJwdWJsaWNLZXkiOiIzWXE4Z3JTNnljVHphQ2pZQWY0UEpEIzEiLCJ0eXBlIjoiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCJ9XSwiaWQiOiIzWXE4Z3JTNnljVHphQ2pZQWY0UEpEIiwicHVibGljS2V5IjpbeyJjb250cm9sbGVyIjoiM1lxOGdyUzZ5Y1R6YUNqWUFmNFBKRCIsImlkIjoiMSIsInB1YmxpY0tleUJhc2U1OCI6IjJQYUNFVW9vUXFlUXpxOGtFZmk4QjhmejNvRkNzbXRDcHl5SDQyTWJLVTlYIiwidHlwZSI6IkVkMjU1MTlWZXJpZmljYXRpb25LZXkyMDE4In1dLCJzZXJ2aWNlIjpbeyJpZCI6ImRpZDpleGFtcGxlOjEyMzQ1Njc4OWFiY2RlZmdoaTtpbmR5IiwicHJpb3JpdHkiOjAsInJlY2lwaWVudEtleXMiOlsiM1lxOGdyUzZ5Y1R6YUNqWUFmNFBKRCMxIl0sInJvdXRpbmdLZXlzIjpbIjNCQWlaenRyRVRmenJWa3hZVTR3S2pUNVZ2WTVWbVVickxBY3lwNmZySjhZIiwiSGV6Y2UyVVdNWjN3VWhWa2gyTGZLU3M4bkR6V3d6czJXaW43RXpOTjNZYVIiXSwic2VydmljZUVuZHBvaW50IjoiaHR0cDovL2xvY2FsaG9zdDo4MDgwL2FnZW5jeS9tc2ciLCJ0eXBlIjoiSW5keUFnZW50In1dfX0=",
        "signature": "W4h3HFIkOu3XHE_QHNfCZZL5t_4ah7zx7UegwyN13P3ugmJVY6UUwYOXrCb0tJL7wEpGKIxguQp21W-e7QQhCg==",
        "signer": "DEKbrMDX9LBGhCk4LBhH6t5B6Kh5iE7GvfepAJYXp7GX"
    },
    "~please_ack": {},
    "~thread": {
        "received_orders": {},
        "sender_order": 0,
        "thid": "b5517062-303f-4267-9a29-09bc89497c06"
    }
}"#;

// Alice sends Ack to Faber
pub const ARIES_CONNECTION_ACK: &str = r#"
{
    "@id": "680e90b0-4a01-4dc7-8a1d-e54b43ebcc28",
    "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/notification/1.0/ack",
    "status": "OK",
    "~thread": {
        "received_orders": {},
        "sender_order": 0,
        "thid": "b5517062-303f-4267-9a29-09bc89497c06"
    }
}"#;

