import { Effect } from 'effect'

import type { AureliaRuntimeContext } from './runtime'

import { logger } from '../lib/logger'
import { runAureliaEffect } from './runtime'

export const runEffectInAppRuntime = <A, E>(
  effect: Effect.Effect<A, E, AureliaRuntimeContext>,
): Promise<A> =>
  runAureliaEffect(effect)

export const runEffectInAppRuntimeLogged = <A, E>(
  effect: Effect.Effect<A, E, AureliaRuntimeContext>,
  context: string,
): Promise<A | undefined> =>
  runAureliaEffect(effect).catch(error => {
    logger.error(`${context}: effect failed`, error)
    return undefined
  })
