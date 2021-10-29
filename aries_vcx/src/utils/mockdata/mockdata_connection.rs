// Alice receives invitation via out of band channel
pub const ARIES_CONNECTION_INVITATION: &str = r#"
{
    "@id": "28b39b79-f5db-4478-879a-15bb12632d00",
    "@type": "https://didcomm.org/connections/1.0/invitation",
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

// Alice created and serialized connection created from received invitation
pub const CONNECTION_SM_INVITEE_INVITED: &str = r#"
{
    "version": "1.0",
    "source_id": "alice-670c6360-5c0e-4495-bd25-2ee58c39fc7e",
    "thread_id": "b5517062-303f-4267-9a29-09bc89497c06",
    "data": {
        "pw_did": "",
        "pw_vk": "",
        "agent_did": "",
        "agent_vk": ""
    },
    "state": {
        "Invitee": {
            "Invited": {
                "invitation": {
                    "@id": "18ac5f5d-c81d-451a-be20-a0df4933513a",
                    "label": "alice-131bc1e2-fa29-404c-a87c-69983e02084d",
                    "recipientKeys": [
                        "HoNSv4aPCRQ8BsJrVXS26Za4rdEFvtCyyoQEtCS175dw"
                    ],
                    "routingKeys": [
                        "DekjTLFWUPs4EPg6tki78Dd99jWnr1JaNMwEgvjAiCMr",
                        "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                    ],
                    "serviceEndpoint": "http://localhost:8080/agency/msg"
                }
            }
        }
    }
}"#;


// Alice sends connection request to Faber
pub const ARIES_CONNECTION_REQUEST: &str = r#"
{
    "@id": "b5517062-303f-4267-9a29-09bc89497c06",
    "@type": "https://didcomm.org/connections/1.0/request",
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

// Alice sends connection request to Faber
pub const CONNECTION_SM_INVITEE_REQUESTED: &str = r#"
{
    "version": "1.0",
    "source_id": "alice-670c6360-5c0e-4495-bd25-2ee58c39fc7e",
    "thread_id": "b5517062-303f-4267-9a29-09bc89497c06",
    "data": {
        "pw_did": "KC6NKcpXcpVnpjL8uKH3tV",
        "pw_vk": "Av4ZDAKgpniTnxLukLQFZ2DbdNqPub8MBxxynCZ5VuFi",
        "agent_did": "Gqw6t57yDgzaG79h4HUVCf",
        "agent_vk": "9drH4FZk79Y4bx5jzPBaJEmB4woEGG1XQSfgF7NkyKvV"
    },
    "state": {
        "Invitee": {
            "Requested": {
                "request": {
                    "@id": "8b58c65b-a585-4976-99e1-f9570a4bd097",
                    "label": "alice-670c6360-5c0e-4495-bd25-2ee58c39fc7e",
                    "connection": {
                        "DID": "KC6NKcpXcpVnpjL8uKH3tV",
                        "DIDDoc": {
                            "@context": "https://w3id.org/did/v1",
                            "id": "KC6NKcpXcpVnpjL8uKH3tV",
                            "publicKey": [
                                {
                                    "id": "1",
                                    "type": "Ed25519VerificationKey2018",
                                    "controller": "KC6NKcpXcpVnpjL8uKH3tV",
                                    "publicKeyBase58": "Av4ZDAKgpniTnxLukLQFZ2DbdNqPub8MBxxynCZ5VuFi"
                                }
                            ],
                            "authentication": [
                                {
                                    "type": "Ed25519SignatureAuthentication2018",
                                    "publicKey": "KC6NKcpXcpVnpjL8uKH3tV#1"
                                }
                            ],
                            "service": [
                                {
                                    "id": "did:example:123456789abcdefghi;indy",
                                    "type": "IndyAgent",
                                    "priority": 0,
                                    "recipientKeys": [
                                        "KC6NKcpXcpVnpjL8uKH3tV#1"
                                    ],
                                    "routingKeys": [
                                        "9drH4FZk79Y4bx5jzPBaJEmB4woEGG1XQSfgF7NkyKvV",
                                        "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                                    ],
                                    "serviceEndpoint": "http://localhost:8080/agency/msg"
                                }
                            ]
                        }
                    }
                },
                "did_doc": {
                    "@context": "https://w3id.org/did/v1",
                    "id": "18ac5f5d-c81d-451a-be20-a0df4933513a",
                    "publicKey": [
                        {
                            "id": "1",
                            "type": "Ed25519VerificationKey2018",
                            "controller": "18ac5f5d-c81d-451a-be20-a0df4933513a",
                            "publicKeyBase58": "HoNSv4aPCRQ8BsJrVXS26Za4rdEFvtCyyoQEtCS175dw"
                        }
                    ],
                    "authentication": [
                        {
                            "type": "Ed25519SignatureAuthentication2018",
                            "publicKey": "18ac5f5d-c81d-451a-be20-a0df4933513a#1"
                        }
                    ],
                    "service": [
                        {
                            "id": "did:example:123456789abcdefghi;indy",
                            "type": "IndyAgent",
                            "priority": 0,
                            "recipientKeys": [
                                "18ac5f5d-c81d-451a-be20-a0df4933513a#1"
                            ],
                            "routingKeys": [
                                "DekjTLFWUPs4EPg6tki78Dd99jWnr1JaNMwEgvjAiCMr",
                                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                            ],
                            "serviceEndpoint": "http://localhost:8080/agency/msg"
                        }
                    ]
                }
            }
        }
    }
}"#;

// Faber sends connection response to Alice, using thid value as was @id in connection request
pub const ARIES_CONNECTION_RESPONSE: &str = r#"
{
    "@id": "586c54a1-8fcf-4539-aebe-19bf02567653",
    "@type": "https://didcomm.org/connections/1.0/response",
    "connection~sig": {
        "@type": "https://didcomm.org/signature/1.0/ed25519Sha512_single",
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

// Alice (invitee) connection SM after Faber accepted connection by sending connection response
pub const CONNECTION_SM_INVITEE_COMPLETED: &str = r#"
{
    "version": "1.0",
    "source_id": "alice-670c6360-5c0e-4495-bd25-2ee58c39fc7e",
    "thread_id": "b5517062-303f-4267-9a29-09bc89497c06",
    "data": {
        "pw_did": "KC6NKcpXcpVnpjL8uKH3tV",
        "pw_vk": "Av4ZDAKgpniTnxLukLQFZ2DbdNqPub8MBxxynCZ5VuFi",
        "agent_did": "Gqw6t57yDgzaG79h4HUVCf",
        "agent_vk": "9drH4FZk79Y4bx5jzPBaJEmB4woEGG1XQSfgF7NkyKvV"
    },
    "state": {
        "Invitee": {
            "Completed": {
                "did_doc": {
                    "@context": "https://w3id.org/did/v1",
                    "id": "2ZHFFhzA2XtTD6hJqzL7ux",
                    "publicKey": [
                        {
                            "id": "1",
                            "type": "Ed25519VerificationKey2018",
                            "controller": "2ZHFFhzA2XtTD6hJqzL7ux",
                            "publicKeyBase58": "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj"
                        }
                    ],
                    "authentication": [
                        {
                            "type": "Ed25519SignatureAuthentication2018",
                            "publicKey": "2ZHFFhzA2XtTD6hJqzL7ux#1"
                        }
                    ],
                    "service": [
                        {
                            "id": "did:example:123456789abcdefghi;indy",
                            "type": "IndyAgent",
                            "priority": 0,
                            "recipientKeys": [
                                "2ZHFFhzA2XtTD6hJqzL7ux#1"
                            ],
                            "routingKeys": [
                                "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2",
                                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                            ],
                            "serviceEndpoint": "http://localhost:8080/agency/msg"
                        }
                    ]
                },
                "bootstrap_did_doc": {
                    "@context": "https://w3id.org/did/v1",
                    "id": "18ac5f5d-c81d-451a-be20-a0df4933513a",
                    "publicKey": [
                        {
                            "id": "1",
                            "type": "Ed25519VerificationKey2018",
                            "controller": "18ac5f5d-c81d-451a-be20-a0df4933513a",
                            "publicKeyBase58": "HoNSv4aPCRQ8BsJrVXS26Za4rdEFvtCyyoQEtCS175dw"
                        }
                    ],
                    "authentication": [
                        {
                            "type": "Ed25519SignatureAuthentication2018",
                            "publicKey": "18ac5f5d-c81d-451a-be20-a0df4933513a#1"
                        }
                    ],
                    "service": [
                        {
                            "id": "did:example:123456789abcdefghi;indy",
                            "type": "IndyAgent",
                            "priority": 0,
                            "recipientKeys": [
                                "18ac5f5d-c81d-451a-be20-a0df4933513a#1"
                            ],
                            "routingKeys": [
                                "DekjTLFWUPs4EPg6tki78Dd99jWnr1JaNMwEgvjAiCMr",
                                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                            ],
                            "serviceEndpoint": "http://localhost:8080/agency/msg"
                        }
                    ]
                },
                "protocols": null,
                "thread_id": "b5517062-303f-4267-9a29-09bc89497c06"
            }
        }
    }
}"#;

// Alice sends Ack to Faber
pub const ARIES_CONNECTION_ACK: &str = r#"
{
    "@id": "680e90b0-4a01-4dc7-8a1d-e54b43ebcc28",
    "@type": "https://didcomm.org/notification/1.0/ack",
    "status": "OK",
    "~thread": {
        "received_orders": {},
        "sender_order": 0,
        "thid": "b5517062-303f-4267-9a29-09bc89497c06"
    }
}"#;

// Inviter (Faber) after finished connection protocol by sending connection ack
pub const CONNECTION_SM_INVITER_COMPLETED: &str = r#"
{
    "version": "1.0",
    "source_id": "alice-131bc1e2-fa29-404c-a87c-69983e02084d",
    "thread_id": "b5517062-303f-4267-9a29-09bc89497c06",
    "data": {
        "pw_did": "2ZHFFhzA2XtTD6hJqzL7ux",
        "pw_vk": "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj",
        "agent_did": "EZrZyu4bfydm4ByNm56kPP",
        "agent_vk": "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2"
    },
    "state": {
        "Inviter": {
            "Completed": {
                "did_doc": {
                    "@context": "https://w3id.org/did/v1",
                    "id": "KC6NKcpXcpVnpjL8uKH3tV",
                    "publicKey": [
                        {
                            "id": "1",
                            "type": "Ed25519VerificationKey2018",
                            "controller": "KC6NKcpXcpVnpjL8uKH3tV",
                            "publicKeyBase58": "Av4ZDAKgpniTnxLukLQFZ2DbdNqPub8MBxxynCZ5VuFi"
                        }
                    ],
                    "authentication": [
                        {
                            "type": "Ed25519SignatureAuthentication2018",
                            "publicKey": "KC6NKcpXcpVnpjL8uKH3tV#1"
                        }
                    ],
                    "service": [
                        {
                            "id": "did:example:123456789abcdefghi;indy",
                            "type": "IndyAgent",
                            "priority": 0,
                            "recipientKeys": [
                                "KC6NKcpXcpVnpjL8uKH3tV#1"
                            ],
                            "routingKeys": [
                                "9drH4FZk79Y4bx5jzPBaJEmB4woEGG1XQSfgF7NkyKvV",
                                "Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR"
                            ],
                            "serviceEndpoint": "http://localhost:8080/agency/msg"
                        }
                    ]
                },
                "protocols": null,
                "thread_id": "b5517062-303f-4267-9a29-09bc89497c06"
            }
        }
    }
}"#;

pub const DEFAULT_SERIALIZED_CONNECTION: &str = r#"
{
  "version": "1.0",
  "source_id": "test_serialize_deserialize",
  "data": {
    "pw_did": "",
    "pw_vk": "",
    "agent_did": "",
    "agent_vk": ""
  },
  "state": {
    "Inviter": {
      "Initial": {}
    }
  }
}"#;

