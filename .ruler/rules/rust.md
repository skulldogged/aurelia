# Rust Guidelines (aurelia-core)

Shared Rust library in `crates/aurelia-core/`, used by the local desktop backend (Electron) and exposed to Android/iOS via uniffi.

## Module Organization

```
crates/aurelia-core/src/
├── lib.rs           # uniffi exports (public API)
├── error/           # Error types (AppError)
├── models/          # Data structures (Song, Credentials, etc.)
├── services/        # External API clients (Jellyfin, LrcLib)
├── db/              # Database schema & repositories
├── cache.rs         # Local caching logic
├── database.rs      # Database connection management
└── utils/           # Helpers (lyrics parsing, etc.)
```

## Error Handling

Use `AppError` for all uniffi-exported functions. Defined in `error/mod.rs`:

```rust
use crate::error::AppError;

// Public API functions return Result<T, AppError>
pub fn my_function() -> Result<MyType, AppError> {
    // Use ? with From implementations
    let data = some_fallible_op()?;
    Ok(data)
}
```

**Error variants**: `Network`, `Auth`, `Database`, `Serialization`, `FileSystem`, `ApiParse`, `Config`, `Http`, `NotFound`, `General`, `UniFfi`

**Internal code**: Use `anyhow::Result` for ergonomic error handling, convert to `AppError` at API boundaries.

## Async Patterns

- Runtime: **Tokio** (multi-threaded)
- uniffi async: Use `#[uniffi::export(async_runtime = "tokio")]`

```rust
#[uniffi::export(async_runtime = "tokio")]
pub async fn fetch_data(url: String) -> Result<Data, AppError> {
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await?;
    Ok(response.json().await?)
}
```

## uniffi Exports

All public API is in `lib.rs`. Functions must be annotated:

```rust
#[uniffi::export]                              // Sync functions
#[uniffi::export(async_runtime = "tokio")]     // Async functions
```

Types exposed to FFI need:
```rust
#[derive(uniffi::Record)]   // For structs
#[derive(uniffi::Enum)]     // For enums
#[derive(uniffi::Error)]    // For error types
```

See [uniffi workflow](./uniffi.md) for binding generation.

## Coding Standards

- **Dependencies**: Prefer `reqwest` for HTTP, `serde` for serialization, `redb` for local storage
- **Logging**: Use `tracing` macros (`tracing::info!`, `tracing::warn!`, etc.)
- **Strings**: Accept `String` (not `&str`) in uniffi exports for FFI compatibility
- **Options**: Use `Option<T>` freely - uniffi handles nullable types
