import { Data } from 'effect'

export type AureliaEffectError = ApiError | PlatformError | StorageError

export class ApiError extends Data.TaggedError('ApiError')<{
  cause?:    unknown
  message:   string
  operation: string
}> {}

export class PlatformError extends Data.TaggedError('PlatformError')<{
  cause?:    unknown
  message:   string
  operation: string
}> {}

export class StorageError extends Data.TaggedError('StorageError')<{
  cause?:    unknown
  key:       string
  message:   string
  operation: 'get' | 'remove' | 'set'
}> {}

export const toErrorMessage = (error: unknown): string => {
  if (typeof error === 'string') {
    return error
  }

  if (error instanceof Error) {
    return error.message
  }

  return String(error)
}
