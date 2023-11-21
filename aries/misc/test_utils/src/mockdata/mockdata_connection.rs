pub const CONNECTION_SM_INVITEE_INVITED: &str = r#"
{
  "connection_sm": {
    "Invitee": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "2GB7BV5cTaXBYC8mrYthU4",
        "pw_vk": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
      },
      "state": {
        "Invited": {
          "invitation": {
            "@id": "testid",
            "label": "",
            "recipientKeys": [
              "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
            ],
            "routingKeys": [],
            "serviceEndpoint": "https://service-endpoint.org"
          },
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "testid",
            "publicKey": [
              {
                "id": "testid#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "testid",
                "publicKeyBase58": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "testid#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          }
        }
      }
    }
  }
}
"#;

pub const CONNECTION_SM_INVITEE_REQUESTED: &str = r#"
{
  "connection_sm": {
    "Invitee": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "2GB7BV5cTaXBYC8mrYthU4",
        "pw_vk": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
      },
      "state": {
        "Requested": {
          "request": {
            "@id": "testid",
            "label": "",
            "connection": {
              "DID": "2GB7BV5cTaXBYC8mrYthU4",
              "DIDDoc": {
                "@context": "https://w3id.org/did/v1",
                "id": "2GB7BV5cTaXBYC8mrYthU4",
                "publicKey": [
                  {
                    "id": "2GB7BV5cTaXBYC8mrYthU4#1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "2GB7BV5cTaXBYC8mrYthU4",
                    "publicKeyBase58": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                  }
                ],
                "authentication": [
                  {
                    "type": "Ed25519SignatureAuthentication2018",
                    "publicKey": "2GB7BV5cTaXBYC8mrYthU4#1"
                  }
                ],
                "service": [
                  {
                    "id": "did:example:123456789abcdefghi;indy",
                    "type": "IndyAgent",
                    "priority": 0,
                    "recipientKeys": [
                      "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                    ],
                    "routingKeys": [],
                    "serviceEndpoint": "https://service-endpoint.org"
                  }
                ]
              }
            },
            "~thread": {
              "thid": "testid",
              "sender_order": 0,
              "received_orders": {}
            },
            "~timing": {
              "out_time": "2023-01-04T10:57:55.405Z"
            }
          },
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "testid",
            "publicKey": [
              {
                "id": "testid#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "testid",
                "publicKeyBase58": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "testid#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          }
        }
      }
    }
  }
}
"#;

pub const CONNECTION_SM_INVITEE_RESPONDED: &str = r#"
{
  "connection_sm": {
    "Invitee": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "2GB7BV5cTaXBYC8mrYthU4",
        "pw_vk": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
      },
      "state": {
        "Responded": {
          "response": {
            "@id": "testid",
            "connection": {
              "DID": "N5bx22gGi4tYSP1kQy9EKW",
              "DIDDoc": {
                "@context": "https://w3id.org/did/v1",
                "id": "N5bx22gGi4tYSP1kQy9EKW",
                "publicKey": [
                  {
                    "id": "N5bx22gGi4tYSP1kQy9EKW#1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "N5bx22gGi4tYSP1kQy9EKW",
                    "publicKeyBase58": "CVMssu7fsjZbt2SDbVQcgmk2EcHHQmC8fqtgnxXkqWR7"
                  }
                ],
                "authentication": [
                  {
                    "type": "Ed25519SignatureAuthentication2018",
                    "publicKey": "N5bx22gGi4tYSP1kQy9EKW#1"
                  }
                ],
                "service": [
                  {
                    "id": "did:example:123456789abcdefghi;indy",
                    "type": "IndyAgent",
                    "priority": 0,
                    "recipientKeys": [
                      "CVMssu7fsjZbt2SDbVQcgmk2EcHHQmC8fqtgnxXkqWR7"
                    ],
                    "routingKeys": [],
                    "serviceEndpoint": "https://service-endpoint.org"
                  }
                ]
              }
            },
            "~please_ack": {},
            "~thread": {
              "thid": "testid",
              "sender_order": 0,
              "received_orders": {}
            },
            "~timing": {
              "out_time": "2023-01-04T10:57:55.408Z"
            }
          },
          "request": {
            "@id": "testid",
            "label": "",
            "connection": {
              "DID": "2GB7BV5cTaXBYC8mrYthU4",
              "DIDDoc": {
                "@context": "https://w3id.org/did/v1",
                "id": "2GB7BV5cTaXBYC8mrYthU4",
                "publicKey": [
                  {
                    "id": "2GB7BV5cTaXBYC8mrYthU4#1",
                    "type": "Ed25519VerificationKey2018",
                    "controller": "2GB7BV5cTaXBYC8mrYthU4",
                    "publicKeyBase58": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                  }
                ],
                "authentication": [
                  {
                    "type": "Ed25519SignatureAuthentication2018",
                    "publicKey": "2GB7BV5cTaXBYC8mrYthU4#1"
                  }
                ],
                "service": [
                  {
                    "id": "did:example:123456789abcdefghi;indy",
                    "type": "IndyAgent",
                    "priority": 0,
                    "recipientKeys": [
                      "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                    ],
                    "routingKeys": [],
                    "serviceEndpoint": "https://service-endpoint.org"
                  }
                ]
              }
            },
            "~thread": {
              "thid": "testid",
              "sender_order": 0,
              "received_orders": {}
            },
            "~timing": {
              "out_time": "2023-01-04T10:57:55.405Z"
            }
          },
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "testid",
            "publicKey": [
              {
                "id": "testid#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "testid",
                "publicKeyBase58": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "testid#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          }
        }
      }
    }
  }
}
"#;

pub const CONNECTION_SM_INVITEE_COMPLETED: &str = r#"
{
  "connection_sm": {
    "Inviter": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "N5bx22gGi4tYSP1kQy9EKW",
        "pw_vk": "CVMssu7fsjZbt2SDbVQcgmk2EcHHQmC8fqtgnxXkqWR7"
      },
      "state": {
        "Responded": {
          "signed_response": {
            "@id": "testid",
            "~thread": {
              "thid": "testid",
              "sender_order": 0,
              "received_orders": {}
            },
            "connection~sig": {
              "@type": "https://didcomm.org/signature/1.0/ed25519Sha512_single",
              "signature": "jDjmdtJ-98B06IUJaS1UH6e0QHFMvlsLcPbgx8-9f8beJ60-VFgOux5UJp02WYvObADvBHY-u240qQ6Qwh5FDw==",
              "sig_data": "AAAAAGO1W7N7IkRJRCI6Ik41YngyMmdHaTR0WVNQMWtReTlFS1ciLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImF1dGhlbnRpY2F0aW9uIjpbeyJwdWJsaWNLZXkiOiJONWJ4MjJnR2k0dFlTUDFrUXk5RUtXIzEiLCJ0eXBlIjoiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCJ9XSwiaWQiOiJONWJ4MjJnR2k0dFlTUDFrUXk5RUtXIiwicHVibGljS2V5IjpbeyJjb250cm9sbGVyIjoiTjVieDIyZ0dpNHRZU1Axa1F5OUVLVyIsImlkIjoiTjVieDIyZ0dpNHRZU1Axa1F5OUVLVyMxIiwicHVibGljS2V5QmFzZTU4IjoiQ1ZNc3N1N2ZzalpidDJTRGJWUWNnbWsyRWNISFFtQzhmcXRnbnhYa3FXUjciLCJ0eXBlIjoiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgifV0sInNlcnZpY2UiOlt7ImlkIjoiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpO2luZHkiLCJwcmlvcml0eSI6MCwicmVjaXBpZW50S2V5cyI6WyJDVk1zc3U3ZnNqWmJ0MlNEYlZRY2dtazJFY0hIUW1DOGZxdGdueFhrcVdSNyJdLCJyb3V0aW5nS2V5cyI6W10sInNlcnZpY2VFbmRwb2ludCI6Imh0dHBzOi8vc2VydmljZS1lbmRwb2ludC5vcmciLCJ0eXBlIjoiSW5keUFnZW50In1dfX0=",
              "signer": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
            },
            "~please_ack": {},
            "~timing": {
              "out_time": "2023-01-04T10:57:55.408Z"
            }
          },
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "2GB7BV5cTaXBYC8mrYthU4",
            "publicKey": [
              {
                "id": "2GB7BV5cTaXBYC8mrYthU4#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "2GB7BV5cTaXBYC8mrYthU4",
                "publicKeyBase58": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "2GB7BV5cTaXBYC8mrYthU4#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          }
        }
      }
    }
  }
}
"#;

pub const CONNECTION_SM_INVITER_REQUESTED: &str = r#"
{
  "connection_sm": {
    "Inviter": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "N5bx22gGi4tYSP1kQy9EKW",
        "pw_vk": "CVMssu7fsjZbt2SDbVQcgmk2EcHHQmC8fqtgnxXkqWR7"
      },
      "state": {
        "Requested": {
          "signed_response": {
            "@id": "testid",
            "~thread": {
              "thid": "testid",
              "sender_order": 0,
              "received_orders": {}
            },
            "connection~sig": {
              "@type": "https://didcomm.org/signature/1.0/ed25519Sha512_single",
              "signature": "jDjmdtJ-98B06IUJaS1UH6e0QHFMvlsLcPbgx8-9f8beJ60-VFgOux5UJp02WYvObADvBHY-u240qQ6Qwh5FDw==",
              "sig_data": "AAAAAGO1W7N7IkRJRCI6Ik41YngyMmdHaTR0WVNQMWtReTlFS1ciLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImF1dGhlbnRpY2F0aW9uIjpbeyJwdWJsaWNLZXkiOiJONWJ4MjJnR2k0dFlTUDFrUXk5RUtXIzEiLCJ0eXBlIjoiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCJ9XSwiaWQiOiJONWJ4MjJnR2k0dFlTUDFrUXk5RUtXIiwicHVibGljS2V5IjpbeyJjb250cm9sbGVyIjoiTjVieDIyZ0dpNHRZU1Axa1F5OUVLVyIsImlkIjoiTjVieDIyZ0dpNHRZU1Axa1F5OUVLVyMxIiwicHVibGljS2V5QmFzZTU4IjoiQ1ZNc3N1N2ZzalpidDJTRGJWUWNnbWsyRWNISFFtQzhmcXRnbnhYa3FXUjciLCJ0eXBlIjoiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgifV0sInNlcnZpY2UiOlt7ImlkIjoiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpO2luZHkiLCJwcmlvcml0eSI6MCwicmVjaXBpZW50S2V5cyI6WyJDVk1zc3U3ZnNqWmJ0MlNEYlZRY2dtazJFY0hIUW1DOGZxdGdueFhrcVdSNyJdLCJyb3V0aW5nS2V5cyI6W10sInNlcnZpY2VFbmRwb2ludCI6Imh0dHBzOi8vc2VydmljZS1lbmRwb2ludC5vcmciLCJ0eXBlIjoiSW5keUFnZW50In1dfX0=",
              "signer": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
            },
            "~please_ack": {},
            "~timing": {
              "out_time": "2023-01-04T10:57:55.408Z"
            }
          },
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "2GB7BV5cTaXBYC8mrYthU4",
            "publicKey": [
              {
                "id": "2GB7BV5cTaXBYC8mrYthU4#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "2GB7BV5cTaXBYC8mrYthU4",
                "publicKeyBase58": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "2GB7BV5cTaXBYC8mrYthU4#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          },
          "thread_id": "testid"
        }
      }
    }
  }
}
"#;

pub const CONNECTION_SM_INVITER_RESPONDED: &str = r#"
{
  "connection_sm": {
    "Inviter": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "N5bx22gGi4tYSP1kQy9EKW",
        "pw_vk": "CVMssu7fsjZbt2SDbVQcgmk2EcHHQmC8fqtgnxXkqWR7"
      },
      "state": {
        "Responded": {
          "signed_response": {
            "@id": "testid",
            "~thread": {
              "thid": "testid",
              "sender_order": 0,
              "received_orders": {}
            },
            "connection~sig": {
              "@type": "https://didcomm.org/signature/1.0/ed25519Sha512_single",
              "signature": "jDjmdtJ-98B06IUJaS1UH6e0QHFMvlsLcPbgx8-9f8beJ60-VFgOux5UJp02WYvObADvBHY-u240qQ6Qwh5FDw==",
              "sig_data": "AAAAAGO1W7N7IkRJRCI6Ik41YngyMmdHaTR0WVNQMWtReTlFS1ciLCJESUREb2MiOnsiQGNvbnRleHQiOiJodHRwczovL3czaWQub3JnL2RpZC92MSIsImF1dGhlbnRpY2F0aW9uIjpbeyJwdWJsaWNLZXkiOiJONWJ4MjJnR2k0dFlTUDFrUXk5RUtXIzEiLCJ0eXBlIjoiRWQyNTUxOVNpZ25hdHVyZUF1dGhlbnRpY2F0aW9uMjAxOCJ9XSwiaWQiOiJONWJ4MjJnR2k0dFlTUDFrUXk5RUtXIiwicHVibGljS2V5IjpbeyJjb250cm9sbGVyIjoiTjVieDIyZ0dpNHRZU1Axa1F5OUVLVyIsImlkIjoiTjVieDIyZ0dpNHRZU1Axa1F5OUVLVyMxIiwicHVibGljS2V5QmFzZTU4IjoiQ1ZNc3N1N2ZzalpidDJTRGJWUWNnbWsyRWNISFFtQzhmcXRnbnhYa3FXUjciLCJ0eXBlIjoiRWQyNTUxOVZlcmlmaWNhdGlvbktleTIwMTgifV0sInNlcnZpY2UiOlt7ImlkIjoiZGlkOmV4YW1wbGU6MTIzNDU2Nzg5YWJjZGVmZ2hpO2luZHkiLCJwcmlvcml0eSI6MCwicmVjaXBpZW50S2V5cyI6WyJDVk1zc3U3ZnNqWmJ0MlNEYlZRY2dtazJFY0hIUW1DOGZxdGdueFhrcVdSNyJdLCJyb3V0aW5nS2V5cyI6W10sInNlcnZpY2VFbmRwb2ludCI6Imh0dHBzOi8vc2VydmljZS1lbmRwb2ludC5vcmciLCJ0eXBlIjoiSW5keUFnZW50In1dfX0=",
              "signer": "FTf8juor9EQSwL4RDHyVdSVuJtLSXzmVZX5fBAmMyH6V"
            },
            "~please_ack": {},
            "~timing": {
              "out_time": "2023-01-04T10:57:55.408Z"
            }
          },
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "2GB7BV5cTaXBYC8mrYthU4",
            "publicKey": [
              {
                "id": "2GB7BV5cTaXBYC8mrYthU4#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "2GB7BV5cTaXBYC8mrYthU4",
                "publicKeyBase58": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "2GB7BV5cTaXBYC8mrYthU4#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          }
        }
      }
    }
  }
}
"#;

pub const CONNECTION_SM_INVITER_COMPLETED: &str = r#"
{
  "connection_sm": {
    "Inviter": {
      "source_id": "",
      "thread_id": "testid",
      "pairwise_info": {
        "pw_did": "N5bx22gGi4tYSP1kQy9EKW",
        "pw_vk": "CVMssu7fsjZbt2SDbVQcgmk2EcHHQmC8fqtgnxXkqWR7"
      },
      "state": {
        "Completed": {
          "did_doc": {
            "@context": "https://w3id.org/did/v1",
            "id": "2GB7BV5cTaXBYC8mrYthU4",
            "publicKey": [
              {
                "id": "2GB7BV5cTaXBYC8mrYthU4#1",
                "type": "Ed25519VerificationKey2018",
                "controller": "2GB7BV5cTaXBYC8mrYthU4",
                "publicKeyBase58": "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
              }
            ],
            "authentication": [
              {
                "type": "Ed25519SignatureAuthentication2018",
                "publicKey": "2GB7BV5cTaXBYC8mrYthU4#1"
              }
            ],
            "service": [
              {
                "id": "did:example:123456789abcdefghi;indy",
                "type": "IndyAgent",
                "priority": 0,
                "recipientKeys": [
                  "gtBaNGsGmZFoDNPeJmPNfncdWoXMWCLCgicJJUZxoGz"
                ],
                "routingKeys": [],
                "serviceEndpoint": "https://service-endpoint.org"
              }
            ]
          },
          "protocols": null,
          "thread_id": "testid"
        }
      }
    }
  }
}
"#;
