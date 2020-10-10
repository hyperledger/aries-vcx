const sleepPromise = require('sleep-promise')
const axios = require('axios')

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

module.exports.buildRevocationDetails = function buildRevocationDetails ({ supportRevocation, tailsFile, maxCreds }) {
  if (supportRevocation === true) {
    return {
      supportRevocation,
      tailsFile,
      maxCreds
    }
  } else {
    return {
      supportRevocation: false
      // tailsFile: '/tmp/tails', // todo: CredDefinition in node wrapper should not require this when revocation is disabled
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
