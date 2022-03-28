const path = require('path')
const { spawn } = require('child_process')

function run (spawnProcess) {
  const spawnedProcess = spawnProcess()
  spawnedProcess.on('error', function (err) {
    console.error(`Encountered error: ${JSON.stringify(err)}`)
    process.exit(-1)
  })
  spawnedProcess.on('exit', function (code, signal) {
    if (code !== 0) {
      console.error('Process finished with nonzero exit code!')
      process.exit(-1)
    }
  })
  spawnedProcess.stdout.on('data', (data) => {
    console.log(data.toString())
  })

  spawnedProcess.stderr.on('data', (data) => {
    console.error(data.toString())
  })
}

const spawnFaber = () => {
  if (process.env.REVOCATION === 'true') {
    return spawn('node',
      [path.resolve(__dirname, './faber.js'),
        '--expose-invitation-port', 8181,
        '--revocation'
      ]
    )
  } else {
    return spawn('node',
      [path.resolve(__dirname, './faber.js'),
        '--expose-invitation-port', 8181
      ]
    )
  }
}

const spawnAlice = () => {
  return spawn('node',
    [path.resolve(__dirname, './alice.js'),
      '--autofetch-invitation-url', 'http://localhost:8181',
    ]
  )
}

run(spawnFaber)
run(spawnAlice)
