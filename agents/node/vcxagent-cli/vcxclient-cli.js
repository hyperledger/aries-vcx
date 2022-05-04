const { runInteractive } = require('./vcxclient-interactive')
const { runScript } = require('./script-common')

const optionDefinitions = [
  {
    name: 'help',
    alias: 'h',
    type: Boolean,
    description: 'Display this usage guide.'
  },
  {
    name: 'acceptTaa',
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
  },
  {
    name: 'agencyUrl',
    type: String,
    description: 'Url of mediator agency',
    defaultValue: 'http://localhost:8080'
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

function areOptionsValid (_options) {
  return true
}

runScript(optionDefinitions, usage, areOptionsValid, runInteractive)
