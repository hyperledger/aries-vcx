const { initRustapi, protocolTypes } = require('../vcx-workflows')
const { StateType, ProofState, Proof } = require('@absaoss/node-vcx-wrapper')
const sleepPromise = require('sleep-promise')
const { runScript } = require('./script-common')
const { createVcxAgent } = require('../vcx-agent')
const logger = require('./logger')('Faber')
const assert = require('assert')
const uuid = require('uuid')
const express = require('express')
const bodyParser = require('body-parser')
const { getAliceSchemaAttrs, getFaberCredDefName, getFaberProofData } = require('../test/data')

async function runFaber (options) {
  logger.info(`Starting. Revocation enabled=${options.revocation}`)
  await initRustapi(process.env.VCX_LOG_LEVEL || 'vcx=error')
  let faberServer
  let exitcode = 0
  try {
    const agentName = `faber-${uuid.v4()}`
    const vcxClient = await createVcxAgent({
      agentName,
      protocolType: options.protocolType,
      agencyUrl: 'http://localhost:8080',
      seed: '000000000000000000000000Trustee1',
      webhookUrl: `http://localhost:7209/notifications/${agentName}`,
      usePostgresWallet: false,
      logger
    })

    if (process.env.ACCEPT_TAA || false) {
      await vcxClient.acceptTaa()
    }

    const schema = await vcxClient.createSchema()
    const schemaId = await schema.getSchemaId()
    await vcxClient.createCredentialDefinition(schemaId, getFaberCredDefName())

    const { connection: connectionToAlice } = await vcxClient.inviterConnectionCreateAndAccept(agentName, (invitationString) => {
      logger.info('\n\n**invite details**')
      logger.info("**You'll ge queried to paste this data to alice side of the demo. This is invitation to connect.**")
      logger.info("**It's assumed this is obtained by Alice from Faber by some existing secure channel.**")
      logger.info('**Could be on website via HTTPS, QR code scanned at Faber institution, ...**')
      logger.info('\n******************\n\n')
      logger.info(invitationString)
      logger.info('\n\n******************\n\n')

      if (options['expose-invitation-port']) {
        const port = options['expose-invitation-port']
        try {
          const appCallbacks = express()
          appCallbacks.use(bodyParser.json())
          appCallbacks.get('/',
            async function (req, res) {
              res.status(200).send({ invitationString })
            }
          )
          faberServer = appCallbacks.listen(port)
          logger.info(`The invitation is also available on port ${port}`)
        } catch (e) {
          logger.error(`Error trying to expose connection invitation on port ${port}`)
        }
      }
    })

    await vcxClient.credentialIssue({ schemaAttrs: getAliceSchemaAttrs(), credDefName: getFaberCredDefName(), connectionNameReceiver: agentName, revoke: options.revocation })

    logger.info('#19 Create a Proof object')
    const vcxProof = await Proof.create(getFaberProofData(vcxClient.getInstitutionDid()))

    logger.info('#20 Request proof of degree from alice')
    await vcxProof.requestProof(connectionToAlice)

    logger.info('#21 Poll agency and wait for alice to provide proof')
    let proofProtocolState = await vcxProof.updateState()
    logger.debug(`vcxProof = ${JSON.stringify(vcxProof)}`)
    logger.debug(`proofState = ${proofProtocolState}`)
    while (proofProtocolState !== StateType.Accepted) { // even if revoked credential was used, state should in final state be StateType.Accepted
      await sleepPromise(2000)
      proofProtocolState = await vcxProof.updateState()
      logger.info(`proofState=${proofProtocolState}`)
    }

    logger.info('#27 Process the proof provided by alice.')
    const { proofState, proof } = await vcxProof.getProof(connectionToAlice)
    assert(proofState)
    assert(proof)
    logger.info(`proofState = ${JSON.stringify(proofProtocolState)}`)
    logger.info(`vcxProof = ${JSON.stringify(vcxProof)}`)

    logger.info('#28 Check if proof is valid.')
    logger.debug(`Serialized proof ${JSON.stringify(await vcxProof.serialize())}`)
    if (proofState === ProofState.Verified) {
      logger.warn('Proof is verified.')
      if (options.revocation) {
        throw Error('Proof was verified, but was expected to be invalid, because revocation was enabled.')
      }
    } else if (proofState === ProofState.Invalid) {
      logger.warn('Proof verification failed. A credential used to create proof may have been revoked.')
      if (options.revocation === false) {
        throw Error('Proof was invalid, but was expected to be verified. Revocation was not enabled.')
      }
    } else {
      logger.error(`Unexpected proof state '${proofState}'.`)
      process.exit(-1)
    }
  } catch (err) {
    exitcode = -1
    logger.error(`Faber encountered error ${err.message} ${err.stack}`)
  } finally {
    if (faberServer) {
      await faberServer.close()
    }
    logger.info(`Exiting process with code ${exitcode}`)
    process.exit(exitcode)
  }
}

const optionDefinitions = [
  {
    name: 'help',
    alias: 'h',
    type: Boolean,
    description: 'Display this usage guide.'
  },
  {
    name: 'protocolType',
    type: String,
    description: 'Protocol type. Possible values: "1.0" "2.0" "3.0" "4.0". Default is 4.0',
    defaultValue: '4.0'
  },
  {
    name: 'postgresql',
    type: Boolean,
    description: 'If specified, postresql wallet will be used.',
    defaultValue: false
  },
  {
    name: 'revocation',
    type: Boolean,
    description: 'If specified, the issued credential will be revoked',
    defaultValue: false
  },
  {
    name: 'expose-invitation-port',
    type: Number,
    description: 'If specified, invitation will be exposed on this port via HTTP'
  }
]

const usage = [
  {
    header: 'Options',
    optionList: optionDefinitions
  },
  {
    content: 'Project home: {underline https://github.com/AbsaOSS/libvcx}'
  }
]

function areOptionsValid (options) {
  if (!(Object.values(protocolTypes).includes(options.protocolType))) {
    console.error(`Unknown protocol type ${options.protocolType}. Only ${JSON.stringify(Object.values(protocolTypes))} are allowed.`)
    return false
  }
  return true
}
runScript(optionDefinitions, usage, areOptionsValid, runFaber)
