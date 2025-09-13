import { createConsola } from 'consola'

export const logger = createConsola({
  level:     import.meta.env.DEV ? 4 : 2, // debug in dev, warn in prod
  reporters: [
    {
      log: logObj => {
        // Add custom formatting or additional logic here
        const prefix = logObj.type === 'info' ? 'ℹ️' :
          logObj.type === 'warn' ? '⚠️' :
            logObj.type === 'error' ? '❌' :
              logObj.type === 'debug' ? '🔍' : '📝'

        // Format args for better readability
        const args = logObj.args?.map((arg: unknown) => {
          if (typeof arg === 'object' && arg !== null) {
            return JSON.stringify(arg, null, 2)
          }
          return arg
        }) || []

        // Use the appropriate console method
        switch (logObj.type) {
          case 'debug':
            console.debug(`${prefix} [${logObj.tag || 'app'}]`, ...args)
            break
          case 'info':
            console.info(`${prefix} [${logObj.tag || 'app'}]`, ...args)
            break
          case 'warn':
            console.warn(`${prefix} [${logObj.tag || 'app'}]`, ...args)
            break
          case 'error':
            console.error(`${prefix} [${logObj.tag || 'app'}]`, ...args)
            break
          default:
            console.log(`${prefix} [${logObj.tag || 'app'}]`, ...args)
        }
      },
    },
  ],
})

// Convenience methods with context
export const createLogger = (tag: string) => {
  const taggedLogger = logger.withTag(tag)
  return {
    debug:   taggedLogger.debug.bind(taggedLogger),
    info:    taggedLogger.info.bind(taggedLogger),
    warn:    taggedLogger.warn.bind(taggedLogger),
    error:   taggedLogger.error.bind(taggedLogger),
    success: taggedLogger.success.bind(taggedLogger),
  }
}

// Pre-configured loggers for different modules
export const playerLogger = createLogger('player')
export const apiLogger = createLogger('api')
export const uiLogger = createLogger('ui')
export const appLogger = createLogger('app')
