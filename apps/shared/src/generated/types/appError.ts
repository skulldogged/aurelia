/**
 * Application-specific error type using thiserror
 */
export type AppError = { network: string } | { auth: string } | { database: string } | { serialization: string } | { fileSystem: string } | { apiParse: string } | { config: string } | { http: { status: number; detail: string } } | { general: string } | { uniFfi: string }