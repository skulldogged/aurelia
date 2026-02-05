import { Effect, Layer } from 'effect'

export interface TimeService {
  now:   () => number
  sleep: (durationMs: number) => Effect.Effect<void>
}

export class TimeServiceTag extends Effect.Tag('aurelia/TimeService')<
  TimeServiceTag,
  TimeService
>() {}

const makeTimeService = (): TimeService => ({
  now:   () => Date.now(),
  sleep: durationMs => Effect.sleep(durationMs),
})

export const TimeServiceLive = Layer.succeed(TimeServiceTag, makeTimeService())

export const now = Effect.flatMap(TimeServiceTag, service => Effect.succeed(service.now()))

export const sleep = (durationMs: number): Effect.Effect<void, never, TimeServiceTag> =>
  Effect.flatMap(TimeServiceTag, service => service.sleep(durationMs))
