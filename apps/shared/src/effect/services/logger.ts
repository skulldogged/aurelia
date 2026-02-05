import { Effect, Layer } from 'effect'

import { logger } from '../../lib/logger'

export interface LoggerService {
  debug: (...args: ReadonlyArray<unknown>) => Effect.Effect<void>
  error: (...args: ReadonlyArray<unknown>) => Effect.Effect<void>
  info:  (...args: ReadonlyArray<unknown>) => Effect.Effect<void>
  warn:  (...args: ReadonlyArray<unknown>) => Effect.Effect<void>
}

export class Logger extends Effect.Tag('aurelia/Logger')<Logger, LoggerService>() {}

const invokeLogger = (
  logMethod: (...args: ReadonlyArray<unknown>) => void,
  args: ReadonlyArray<unknown>,
): void => {
  if (args.length === 0) {
    logMethod('')
    return
  }

  const [firstArg, ...restArgs] = args
  logMethod(firstArg, ...restArgs)
}

const makeLoggerService = (): LoggerService => ({
  debug: (...args) => Effect.sync(() => invokeLogger(logger.debug, args)),
  error: (...args) => Effect.sync(() => invokeLogger(logger.error, args)),
  info:  (...args) => Effect.sync(() => invokeLogger(logger.info, args)),
  warn:  (...args) => Effect.sync(() => invokeLogger(logger.warn, args)),
})

export const LoggerLive = Layer.succeed(Logger, makeLoggerService())

export const logDebug = (...args: ReadonlyArray<unknown>): Effect.Effect<void, never, Logger> =>
  Effect.flatMap(Logger, service => service.debug(...args))

export const logError = (...args: ReadonlyArray<unknown>): Effect.Effect<void, never, Logger> =>
  Effect.flatMap(Logger, service => service.error(...args))

export const logInfo = (...args: ReadonlyArray<unknown>): Effect.Effect<void, never, Logger> =>
  Effect.flatMap(Logger, service => service.info(...args))

export const logWarn = (...args: ReadonlyArray<unknown>): Effect.Effect<void, never, Logger> =>
  Effect.flatMap(Logger, service => service.warn(...args))
