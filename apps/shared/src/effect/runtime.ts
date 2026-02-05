import { Effect, Layer, ManagedRuntime } from 'effect'

import { LoggerLive } from './services/logger'
import { StorageServiceLive } from './services/storage'
import { TimeServiceLive } from './services/time'

export const AureliaLayer = Layer.mergeAll(
  LoggerLive,
  StorageServiceLive,
  TimeServiceLive,
)

export type AureliaRuntimeContext = Layer.Layer.Success<typeof AureliaLayer>

const aureliaRuntime = ManagedRuntime.make(AureliaLayer)

export const provideAureliaLayer = <A, E, R>(
  effect: Effect.Effect<A, E, R>,
): Effect.Effect<A, E, Exclude<R, AureliaRuntimeContext>> =>
  Effect.provide(effect, AureliaLayer)

export const runAureliaEffect = <A, E>(
  effect: Effect.Effect<A, E, AureliaRuntimeContext>,
): Promise<A> =>
  aureliaRuntime.runPromise(effect)

export const runAureliaEffectExit = <A, E>(
  effect: Effect.Effect<A, E, AureliaRuntimeContext>,
) =>
  aureliaRuntime.runPromiseExit(effect)

export const disposeAureliaRuntime = (): Promise<void> =>
  aureliaRuntime.dispose()
