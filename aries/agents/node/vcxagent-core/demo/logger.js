const { createLogger, format, transports } = require('winston')

const prettyFormatter = format.combine(
  (process.env.DISABLE_COLOR_LOGS !== 'true') ? format.colorize({ all: true }) : format.uncolorize({}),
  format.printf(
    msg => {
      const extras = (global.expect) ? `${global.expect.getState().currentTestName}` : ''
      return `[${msg.timestamp}] [${msg.filename}] [${msg.label}] [${msg.level}] ${extras}: ${msg.message}`
    }
  )
)

module.exports = loggerLabel => {
  return createLogger({
    level: 'debug',
    format: format.combine(
      format.label({ label: loggerLabel }),
      format.timestamp({
        format: 'YYYY-MM-DD HH:mm:ss.SSS'
      }),
      prettyFormatter
    ),
    transports: [
      new transports.Console()
    ]
  })
}
