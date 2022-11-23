const { performance } = require('perf_hooks')
const { OutOfBandSender, GoalCode, HandshakeProtocol, initRustAPI } = require('../dist')
const uuid = require('uuid')
const { initThreadpool } = require('../dist/api/init')
const sleep = require('sleep-promise')

async function timeFunctionExecution (fn, ...args) {
  const startTime = performance.now()
  const res = await fn(...args)
  const endTime = performance.now()
  const durationMs = endTime - startTime
  return { res, durationMs }
}


const credentialOffer = {
  "@id": "57b3f85d-7673-4e6f-bb09-cc27cf2653c0",
  "@type": "https://didcomm.org/issue-credential/1.0/offer-credential",
  "credential_preview": {
    "@type": "https://didcomm.org/issue-credential/1.0/credential-preview",
    "attributes": [
      {
        "name": "age",
        "value": "25"
      },
    ]
  },
  "offers~attach": [
    {
      "@id": "libindy-cred-offer-0",
      "data": {
        "base64": "eyJzY2hlzU0NzA3NTkwOTc1MTUyNTk4MTgwNyJ9"
      },
      "mime-type": "application/json"
    }
  ]
}


async function run() {
  const rustApi = initRustAPI()
  // await rustApi.vcx_set_default_logger('trace')
  await initThreadpool({ num_threads: 4 })

  const startTime = performance.now()
  for (let i=0; i<100; i++) {
    const oobSender = await OutOfBandSender.create({
      source_id: uuid.v4(),
      label: uuid.v4(),
      goalCode: GoalCode.P2PMessaging,
      goal: uuid.v4(),
      handshake_protocols: [HandshakeProtocol.ConnectionV1]
    })
    // await oobSender.appendServiceDid("VsKV7grR1BUE29mG2Fm2kX")
    // const service = {
    //   "id": "did:example:123456789abcdefghi;indy",
    //   "priority": 0,
    //   "recipientKeys": ["abcde", uuid.v4()],
    //   "routingKeys": ["12345", uuid.v4()],
    //   "serviceEndpoint": "http://example.org/agent",
    //   "type": uuid.v4()
    // }
    // await oobSender.appendService(JSON.stringify(service))
    // await oobSender.appendMessage(JSON.stringify(credentialOffer))
    // const msg = oobSender.toMessage()
  }
  const endTime = performance.now()
  const durationMs = endTime - startTime
  console.log(durationMs)
  await sleep(1000)
  // console.log(JSON.stringify(msg))
}

run()
