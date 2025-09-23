import { createConsola } from 'consola'

// Redacts sensitive keys from objects before logging
const sensitiveKeys = ['token', 'password', 'pass', 'apiKey', 'credentials']

const replacer = (key: string, value: unknown) =>
  typeof key === 'string' && sensitiveKeys.includes(key.toLowerCase()) ?
    '******' : value

const levelColors: Record<string, string> = {
  debug: '#89b4fa',
  info:  '#a6e3a1',
  warn:  '#f9e2af',
  error: '#f38ba8',
  log:   '#cdd6f4',
}

export const logger = createConsola({
  level:     import.meta.env.DEV ? 4 : 2, // debug in dev, warn in prod
  reporters: [
    {
      log: logObj => {
        const prefix = logObj.type === 'info' ? '[INFO]' :
          logObj.type === 'warn' ? '[WARN]' :
            logObj.type === 'error' ? '[ERROR]' :
              logObj.type === 'debug' ? '[DEBUG]' : '[LOG]'

        const args = logObj.args?.map((arg: unknown) => {
          if (arg instanceof Error) {
            return arg.stack || arg.message
          }
          if (typeof arg === 'object' && arg !== null) {
            return JSON.stringify(arg, replacer, 2)
          }
          return arg
        }) || []

        const color = levelColors[logObj.type] || levelColors.log
        const style = `color: ${color}; font-weight: bold;`
        const tag = `[${logObj.tag || 'app'}]`

        switch (logObj.type) {
          case 'debug':
            console.debug(`%c${prefix}`, style, tag, ...args)
            break
          case 'info':
            console.info(`%c${prefix}`, style, tag, ...args)
            break
          case 'warn':
            console.warn(`%c${prefix}`, style, tag, ...args)
            break
          case 'error':
            console.error(`%c${prefix}`, style, tag, ...args)
            break
          default:
            console.log(`%c${prefix}`, style, tag, ...args)
        }
      },
    },
  ],
})

// Convenience methods with context
export const createLogger = (tag: string) => ({
  debug: (logger.withTag(tag)).debug.bind(logger.withTag(tag)),
  info:  (logger.withTag(tag)).info.bind(logger.withTag(tag)),
  warn:  (logger.withTag(tag)).warn.bind(logger.withTag(tag)),
  error: (logger.withTag(tag)).error.bind(logger.withTag(tag)),
})

// Pre-configured loggers for different modules
export const playerLogger = createLogger('player')
export const apiLogger = createLogger('api')
export const uiLogger = createLogger('ui')
export const appLogger = createLogger('app')
export const authLogger = createLogger('auth')
