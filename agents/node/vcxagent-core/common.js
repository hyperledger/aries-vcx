const sleepPromise = require('sleep-promise')
const axios = require('axios')

function getRandomInt (min, max) {
  min = Math.ceil(min)
  max = Math.floor(max)
  return Math.floor(Math.random() * (max - min)) + min
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

async function pollFunction (fn, actionDescription, logger, attemptsThreshold = 10, timeout = 2000) {
  let { result, isFinished } = await fn()
  let attempts = 1
  while (!isFinished) {
    if (attempts > attemptsThreshold) {
      const error = `Tried to poll ${attempts} times and result was not received.`
      return [error, null]
    }
    logger.info(`Trying to do: ${actionDescription} Attempt ${attempts}/${attemptsThreshold}. Will try again after ${timeout}ms.`)
    await sleepPromise(timeout);
    ({ result, isFinished } = await fn())
    attempts += 1
  }
  return [null, result]
}

module.exports.getRandomInt = getRandomInt
module.exports.waitUntilAgencyIsReady = waitUntilAgencyIsReady
module.exports.pollFunction = pollFunction
