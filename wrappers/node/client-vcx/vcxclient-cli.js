const { runInteractive } = require('./vcxclient-interactive')
const { runScript } = require('../common/script-comon')

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
    name: 'accepttaa',
    type: Boolean,
    description: 'If specified accpets taa',
    defaultValue: false
  },
  {
    name: 'seed',
    type: String,
    description: 'Provision seed',
    defaultValue: '000000000000000000000000Trustee1'
  },
  {
    name: 'name',
    type: String,
    description: 'Agent name'
  },
  {
    name: 'RUST_LOG',
    type: String,
    description: 'Agent name',
    defaultValue: 'vcx=error'
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
const allowedProtocolTypes = ['1.0', '2.0', '3.0', '4.0']

function areOptionsValid (options) {
  if (!(allowedProtocolTypes.includes(options.protocolType))) {
    console.error(`Unknown protocol type ${options.protocolType}. Only ${JSON.stringify(allowedProtocolTypes)} are allowed.`)
    return false
  }
  if (!options.name) {
    console.error('Must specify --name.')
    return false
  }
  return true
}

runScript(optionDefinitions, usage, areOptionsValid, runInteractive)
