# uniffi Workflow

uniffi generates Kotlin bindings from the Rust `aurelia-core` crate, enabling Android to call shared Rust code.

## How It Works

1. Rust functions/types annotated with `#[uniffi::export]` in `crates/aurelia-core/src/lib.rs`
2. `uniffi-bindgen` generates Kotlin bindings from the compiled library
3. Android loads the native library and calls Rust via JNA

## Regenerating Bindings

From the **project root**:

```bash
# 1. Build Rust library (host target for bindgen)
cargo build -p aurelia-core

# 2. Generate Kotlin bindings
cargo run -p uniffi-bindgen -- generate \
  --library target/debug/aurelia_core.dll \
  --language kotlin \
  --out-dir apps/mobile/android/app/src/main/java \
  --no-format
```

Or use the Gradle task (from `apps/mobile/android`):
```bash
./gradlew generateUniffiBindings
```

## Building for Android

```bash
# Build for Android targets (requires cargo-ndk)
cargo ndk -t arm64-v8a -t x86_64 \
  -o apps/mobile/android/app/src/main/jniLibs \
  build -p aurelia-core --release

# Copy with correct name
./gradlew copyUniffiLibs
```

Or just build the Android app (runs all tasks automatically):
```bash
cd apps/mobile/android && ./gradlew assembleDebug
```

## Adding New Exports

1. Add function in `crates/aurelia-core/src/lib.rs`:
   ```rust
   #[uniffi::export]
   pub fn new_function(arg: String) -> Result<ReturnType, AppError> {
       // implementation
   }
   ```

2. If new types needed, add derives in `models/`:
   ```rust
   #[derive(uniffi::Record, serde::Serialize, serde::Deserialize)]
   pub struct NewType {
       pub field: String,
   }
   ```

3. Regenerate bindings (see above)

4. Use in Kotlin:
   ```kotlin
   import uniffi.aurelia_core.newFunction

   val result = newFunction("arg")
   ```

## Generated Files

- Kotlin: `apps/mobile/android/app/src/main/java/uniffi/aurelia_core/aurelia_core.kt`
- Native libs: `apps/mobile/android/app/src/main/jniLibs/{arch}/libuniffi_aurelia_core.so`

## Troubleshooting

- **"Library not found"**: Run `cargo build -p aurelia-core` first
- **Type mismatch**: Ensure Rust types have correct uniffi derives
- **Async not working**: Verify `async_runtime = "tokio"` annotation
