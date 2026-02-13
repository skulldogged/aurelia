import { createConsola } from 'consola'

// Redacts sensitive keys from objects before logging
const sensitiveKeys = ['token', 'password', 'pass', 'apiKey', 'credentials']

const replacer = (key: string, value: unknown): unknown =>
  typeof key === 'string' && sensitiveKeys.includes(key.toLowerCase()) ?
    '******' : value

const levelColors: Record<string, string> = {
  debug: 'dodgerblue',
  error: 'crimson',
  info:  'limegreen',
  warn:  'gold',
}

const isDev = (import.meta as ImportMeta & { env?: { DEV?: boolean } }).env?.DEV ?? false

export const logger = createConsola({
  level:     isDev ? 4 : 2, // debug in dev, warn in prod
  reporters: [
    {
      log: logObj => {
        const timestamp = new Date().toISOString()
        const level = logObj.type.toUpperCase()

        const error = new Error()
        const stackLines = error.stack?.split('\n') || []
        let tag = logObj.tag || 'unknown'

        // Find the caller line that contains 'src/'
        let callerLine = ''
        for (const line of stackLines.slice(2)) { // Skip Error and log function
          if (line.includes('src/')) {
            callerLine = line
            break
          }
        }

        // Parse file path from caller line
        const match = callerLine.match(/(src\/[^:]+)/)
        if (match) {
          const filePath = match[1]
          const extension = `[${filePath.split('.')[1]?.split('?')[0]}]`
          tag = `${filePath.replace(/^src\//, '').replace(/\..*/, '').replace(/\//g, '::')} ${extension}`
        }

        const args = logObj.args?.map((arg: unknown) =>
          arg instanceof Error
            ?  arg.stack || arg.message
            : typeof arg === 'object' && arg !== null
              ?  JSON.stringify(arg, replacer, 2)
              : arg,
        ) || []

        const message = args.join(' ')
        const color = levelColors[level.toLowerCase()] || '#cdd6f4'
        const levelStr = level === 'DEBUG' || level === 'ERROR' ? `${level}` : ` ${level}`
        const logLine = `%c${timestamp}%c %c${levelStr}%c %caurelia::${tag}:%c ${message}`

        console.log(
          logLine,
          'color: gray',
          '',
          `color: ${color}`,
          '',
          'color: gray',
          '',
        )
      },
    },
  ],
})

