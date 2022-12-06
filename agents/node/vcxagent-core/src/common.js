const sleepPromise = require('sleep-promise')
const axios = require('axios')
module.exports.testTailsUrl = 'http://some-tails-url.org'
if(typeof process != 'undefined'){
  axios.defaults.adapter = require('axios/lib/adapters/http');
}
async function waitUntilAgencyIsReady (agencyEndpoint, logger) {
  let agencyReady = false
  while (!agencyReady) {
    try {
      await axios.get(`${agencyEndpoint}/agency`)
      agencyReady = true
    } catch (e) {
      logger.warn(`Agency ${agencyEndpoint} should return 200OK on HTTP GET ${agencyEndpoint}/agency, but returns error: ${e}. Sleeping.`)
      await sleepPromise(1000)
    }
  }
}

async function getAgencyConfig (agencyUrl, logger) {
  let agencyDid, agencyVerkey
  const agencyInfoPath = `${agencyUrl}/agency`
  logger.info(`Obtaining agency DID and verkey info from ${agencyInfoPath}`)
  try {
    const { data } = await axios.get(agencyInfoPath)
    agencyDid = data.DID
    agencyVerkey = data.verKey
    if (!agencyDid || !agencyVerkey) {
      throw Error(`Agency returned unexpected DID and verkey format: ${JSON.stringify(data)}`)
    }
  } catch (err) {
    logger.warn(`Failed to obtain DID and verkey from agency with error ${err}. Defaults will be used.`)
    agencyDid = 'VsKV7grR1BUE29mG2Fm2kX'
    agencyVerkey = 'Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR'
  }
  return {
    agency_endpoint: agencyUrl,
    agency_did: agencyDid,
    agency_verkey: agencyVerkey
  }
}

async function pollFunction (fn, actionDescription, logger, attemptsThreshold = 10, timeoutMs = 2000) {
  let { result, isFinished } = await fn()
  let attempts = 1
  while (!isFinished) {
    if (attempts > attemptsThreshold) {
      const error = `Tried to poll ${attempts} times and result was not received.`
      return [error, null]
    }
    logger.info(`Trying to do: ${actionDescription} Attempt ${attempts}/${attemptsThreshold}. Will try again after ${timeoutMs}ms.`)
    await sleepPromise(timeoutMs);
    ({ result, isFinished } = await fn())
    attempts += 1
  }
  return [null, result]
}

function getSampleSchemaData () {
  const version = `${getRandomInt(1, 101)}.${getRandomInt(1, 101)}.${getRandomInt(1, 101)}`
  return {
    data: {
      attrNames: ['name', 'last_name', 'sex', 'date', 'degree', 'age'],
      name: 'FaberVcx',
      version
    },
    paymentHandle: 0,
    sourceId: `your-identifier-fabervcx-${version}`
  }
}

module.exports.buildRevocationDetails = function buildRevocationDetails ({ supportRevocation, tailsDir, tailsUrl, maxCreds }) {
  if (supportRevocation === true) {
    return {
      supportRevocation,
      tailsDir,
      tailsUrl,
      maxCreds
    }
  } else {
    return {
      supportRevocation: false
    }
  }
}

function getRandomInt (min, max) {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min)) + min
}

module.exports.waitUntilAgencyIsReady = waitUntilAgencyIsReady
module.exports.pollFunction = pollFunction
module.exports.getSampleSchemaData = getSampleSchemaData
module.exports.getAgencyConfig = getAgencyConfig
