# Mobile Guidelines (Kotlin + Compose)

Covers both Android and iOS platforms. See architecture.md for platform structure.

## Android Architecture

- **UI**: Composables in `ui/` package. One file per screen.
- **State**: `ViewModel` + `StateFlow`. Data classes in same file or `*State.kt`.
- **Logic**: ViewModels handle business logic and call uniffi bindings.
- **Dependency Injection**: Use Factory pattern for ViewModels.
- **Player**: `PlayerController` wraps Media3; `PlayerSnapshot` holds state.
- **Network**:
  - Jellyfin API: Prefer uniffi bindings.
  - External APIs: `data/network/` (OkHttp).
- **Storage**: `SessionStore` for credentials/preferences.

## Coding Standards

- **Components**: Jetpack Compose with Material 3.
- **Async**: Coroutines with `viewModelScope` and `Dispatchers.IO`.
- **Theming**: Use `MaterialTheme.colorScheme`.
- **Reference**: Follow patterns in `apps/mobile/android/app/src/main/java/com/aurelia/app/`.

### UX/Accessibility
- Maintain contentDescription for accessibility.
- Respect reduced motion preferences.

## iOS

Located at `apps/mobile/ios/`. Uses native iOS development with uniffi bindings to shared Rust core (`crates/aurelia-core/`). See uniffi.md for binding generation details.
