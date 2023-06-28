pub static OOB_INVITE: &str = r##"
{
  "@type": "https://didcomm.org/out-of-band/1.1/invitation",
  "@id": "032fbd19-f6fd-48c5-9197-ba9a47040470",
  "label": "Faber College",
  "goal_code": "issue-vc",
  "goal": "To issue a Faber College Graduate credential",
  "accept": [
    "didcomm/aip2;env=rfc587",
    "didcomm/aip2;env=rfc19"
  ],
  "handshake_protocols": [
    "https://didcomm.org/didexchange/1.0",
    "https://didcomm.org/connections/1.0"
  ],
  "requests~attach": [],
  "services": ["did:peer:2.Ez6LSpSrLxbAhg2SHwKk7kwpsH7DM7QjFS5iK6qP87eViohud.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V"]
}
"##;

pub static REQUEST: &str = r##"
{
  "@id": "a46cdd0f-a2ca-4d12-afbf-2e78a6f1f3ef",
  "@type": "https://didcomm.org/didexchange/1.0/request",
  "~thread": {
      "thid": "a46cdd0f-a2ca-4d12-afbf-2e78a6f1f3ef",
      "pthid": "032fbd19-f6fd-48c5-9197-ba9a47040470"
  },
  "label": "Bob",
  "goal_code": "aries.rel.build",
  "goal": "To create a relationship",
  "did": "B.did@B:A",
  "did_doc~attach": {
      "@id": "d2ab6f2b-5646-4de3-8c02-762f553ab804",
      "mime-type": "application/json",
      "data": {
         "base64": "eyJ0eXAiOiJKV1Qi... (bytes omitted)",
         "jws": {
            "header": {
               "kid": "did:key:z6MkmjY8GnV5i9YTDtPETC2uUAW6ejw3nk5mXF5yci5ab7th"
            },
            "protected": "eyJhbGciOiJFZERTQSIsImlhdCI6MTU4Mzg4... (bytes omitted)",
            "signature": "3dZWsuru7QAVFUCtTd0s7uc1peYEijx4eyt5... (bytes omitted)"
            }
      }
   }
}
"##;

pub static RESPONSE: &str = r##"
{
  "@type": "https://didcomm.org/didexchange/1.0/response",
  "@id": "12345678900987654321",
  "~thread": {
    "thid": "<The Thread ID is the Message ID (@id) of the first message in the thread>"
  },
  "did": "B.did@B:A",
  "did_doc~attach": {
      "@id": "d2ab6f2b-5646-4de3-8c02-762f553ab804",
      "mime-type": "application/json",
      "data": {
         "base64": "eyJ0eXAiOiJKV1Qi... (bytes omitted)",
         "jws": {
            "header": {
               "kid": "did:key:z6MkmjY8GnV5i9YTDtPETC2uUAW6ejw3nk5mXF5yci5ab7th"
            },
            "protected": "eyJhbGciOiJFZERTQSIsImlhdCI6MTU4Mzg4... (bytes omitted)",
            "signature": "3dZWsuru7QAVFUCtTd0s7uc1peYEijx4eyt5... (bytes omitted)"
            }
      }
   }
}
"##;

pub static COMPLETE: &str = r##"
{
  "@type": "https://didcomm.org/didexchange/1.0/complete",
  "@id": "12345678900987654321",
  "~thread": {
    "thid": "<The Thread ID is the Message ID (@id) of the first message in the thread>",
    "pthid": "<pthid used in request message>"
  }
}
"##;
