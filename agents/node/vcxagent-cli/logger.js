const { createLogger, format, transports } = require('winston')
const { label } = format

const prettyFormatter = format.combine(
  format.printf(
    info => `${info.label} [${info.level}]: ${info.message}`
  )
)

module.exports = loggerLabel => {
  return createLogger({
    level: 'debug',
    format: format.combine(
      label({ label: loggerLabel }),
      format.colorize({ all: true }),
      prettyFormatter
    ),
    transports: [
      new transports.Console()
    ]
  })
}
