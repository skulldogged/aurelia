# Effect Migration Tracker

This document tracks the incremental migration of shared TypeScript code to Effect.

## Current status

- [x] Phase 1 foundation created in `apps/shared/src/effect`
- [x] Effect dependencies added to `apps/shared/package.json`
- [x] Runtime layer with logger, storage, and time services
- [x] API boundary wrappers introduced for core operations
- [x] `home` store loading path migrated to Effect retry/runtime
- [x] `library` store API flows migrated to Effect runtime wrappers
- [x] `useAuth` and `useSession` backend calls migrated to Effect runtime wrappers
- [x] `playlists`, `useLastFm`, and `useListenBrainz` API flows migrated to Effect runtime wrappers
- [x] `login.vue` authentication flow migrated to Effect runtime wrappers
- [x] `useLibrary`, `useImageLoader`, `useSongInteractions`, and artist page API flows migrated
- [x] `ShareDialog`, `LyricsView`, `MusicPlayer`, `useSystemTray`, and `IntegrationsSettings` migrated
- [x] `useAudioEngine`, `useVisualizerData`, and Rust audio player migrated to Effect wrappers
- [x] Desktop/Web app shell sync-state and quit flows migrated to Effect wrappers
- [x] API codegen updated for array-safe query typing (via macro generator, then regenerated client)
- [x] Stores/composables migrated module-by-module
- [x] Remove dead legacy `Result` helper usage (`apps/shared/src/lib/result.ts` + tests)

## Incremental rollout strategy

1. Keep `apps/shared/src/api/apiClient.ts` as the transport boundary for now.
2. Move store/composable async logic to Effect wrappers in `apps/shared/src/effect/services`.
3. Keep Pinia and Vue interfaces stable while replacing implementation internals.
4. Remove compatibility wrappers only after parity tests are green.

## Next migration targets

1. Add Effect-focused tests for migrated wrappers and high-risk playback/auth flows
2. Evaluate moving selected store state transitions into Effect services for better composability
3. Decide whether to retain or simplify `Option` helpers still used in player/local state logic
