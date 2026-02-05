import { Effect } from 'effect'

import type { Result } from '../../lib/api/result'
import type {
  Artist,
  Credentials,
  HomeViewData,
  LastFmCredentials,
  LibraryData,
  ListenBrainzCredentials,
  ListenBrainzListen,
  NowPlayingPayload,
  Playlist,
  PlaylistCreateData,
  PlaylistUpdateData,
  RpcActivity,
  Song,
  SyncStateInfo,
} from '../../lib/api/types'

import { getApiClient } from '../../index'
import { ApiError, toErrorMessage } from '../errors'

const toApiError = (operation: string, cause: unknown, message?: string): ApiError =>
  new ApiError({
    cause,
    message: message ?? toErrorMessage(cause),
    operation,
  })

export const apiResultToEffect = <T>(
  operation: string,
  result: Result<T>,
): Effect.Effect<T, ApiError> =>
  result.status === 'ok'
    ? Effect.succeed(result.data)
    : Effect.fail(toApiError(operation, result.error, String(result.error)))

const runApiRequest = <T>(
  operation: string,
  request: () => Promise<Result<T>>,
): Effect.Effect<T, ApiError> =>
  Effect.tryPromise({
    catch: cause => toApiError(operation, cause),
    try:   request,
  }).pipe(
    Effect.flatMap(result => apiResultToEffect(operation, result)),
  )

export const clearCacheEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('clearCache', () => getApiClient().clearCache())

export const clearSavedCredentialsEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('clearSavedCredentials', () => getApiClient().clearSavedCredentials())

export const getHomeViewDataEffect = (): Effect.Effect<HomeViewData, ApiError> =>
  runApiRequest<HomeViewData>('getHomeViewData', () => getApiClient().getHomeViewData())

export const getAudioStreamUrlEffect = (
  itemId: string,
  serverUrl: string,
  token: string,
  container?: string,
): Effect.Effect<string, ApiError> =>
  runApiRequest<string>('getAudioStreamUrl', () =>
    getApiClient().getAudioStreamUrl(itemId, serverUrl, token, container),
  )

export const getLyricsEffect = (
  id: string,
  artist: string,
  title: string,
  path?: string,
): Effect.Effect<string, ApiError> =>
  runApiRequest<string>('getLyrics', () => getApiClient().getLyrics(id, artist, title, path))

export const getImageEffect = (
  itemId: string,
  imageType: string,
  serverUrl: string,
  token: string,
  width?: number,
  quality?: number,
): Effect.Effect<string, ApiError> =>
  runApiRequest<string>('getImage', () =>
    getApiClient().getImage(itemId, imageType, serverUrl, token, width, quality),
  )

export const getImageCacheStatsEffect = (): Effect.Effect<string, ApiError> =>
  runApiRequest<string>('getImageCacheStats', () => getApiClient().getImageCacheStats())

export const getLibraryEffect = (): Effect.Effect<LibraryData, ApiError> =>
  runApiRequest<LibraryData>('getLibrary', () => getApiClient().getLibrary())

export const getArtistEffect = (artistId: string): Effect.Effect<Artist, ApiError> =>
  runApiRequest<Artist>('getArtist', () => getApiClient().getArtist(artistId))

export const getRelatedArtistsEffect = (artistId: string): Effect.Effect<Artist[], ApiError> =>
  runApiRequest<Artist[]>('getRelatedArtists', () => getApiClient().getRelatedArtists(artistId))

export const getAlbumShareUrlsEffect = (albumId: string): Effect.Effect<Record<string, string>, ApiError> =>
  runApiRequest<Record<string, string>>('getAlbumShareUrls', () => getApiClient().getAlbumShareUrls(albumId))

export const getArtistShareUrlsEffect = (artistId: string): Effect.Effect<Record<string, string>, ApiError> =>
  runApiRequest<Record<string, string>>('getArtistShareUrls', () => getApiClient().getArtistShareUrls(artistId))

export const getSongShareUrlsEffect = (itemId: string): Effect.Effect<Record<string, string>, ApiError> =>
  runApiRequest<Record<string, string>>('getSongShareUrls', () => getApiClient().getSongShareUrls(itemId))

export const getSavedCredentialsEffect = (): Effect.Effect<Credentials | null, ApiError> =>
  runApiRequest<Credentials | null>('getSavedCredentials', () => getApiClient().getSavedCredentials())

export const getSyncStateEffect = (): Effect.Effect<SyncStateInfo, ApiError> =>
  runApiRequest<SyncStateInfo>('getSyncState', () => getApiClient().getSyncState())

export const getInstantMixEffect = (itemId: string): Effect.Effect<Song[], ApiError> =>
  runApiRequest<Song[]>('getInstantMix', () => getApiClient().getInstantMix(itemId))

export const getPlaylistItemsEffect = (playlistId: string): Effect.Effect<Song[], ApiError> =>
  runApiRequest<Song[]>('getPlaylistItems', () => getApiClient().getPlaylistItems(playlistId))

export const getPlaylistsEffect = (): Effect.Effect<Playlist[], ApiError> =>
  runApiRequest<Playlist[]>('getPlaylists', () => getApiClient().getPlaylists())

export const markItemPlayedEffect = (itemId: string): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('markItemPlayed', () => getApiClient().markItemPlayed(itemId))

export const clearImageFromCacheEffect = (itemId: string, imageType: string): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('clearImageFromCache', () => getApiClient().clearImageFromCache(itemId, imageType))

export const loginToJellyfinEffect = (
  serverUrl: string,
  username: string,
  password: string,
  deviceId: string,
): Effect.Effect<Credentials, ApiError> =>
  runApiRequest<Credentials>('loginToJellyfin', () =>
    getApiClient().loginToJellyfin(serverUrl, username, password, deviceId),
  )

export const saveCredentialsEffect = (
  serverUrl: string,
  username: string,
  token: string,
  userId: string,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('saveCredentials', () =>
    getApiClient().saveCredentials(serverUrl, username, token, userId),
  )

export const syncLibraryEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('syncLibrary', () => getApiClient().syncLibrary())

export const toggleFavoriteStatusEffect = (
  itemId: string,
  isFavorite: boolean,
): Effect.Effect<boolean, ApiError> =>
  runApiRequest<boolean>('toggleFavoriteStatus', () =>
    getApiClient().toggleFavoriteStatus(itemId, isFavorite),
  )

export const createPlaylistEffect = (data: PlaylistCreateData): Effect.Effect<Playlist, ApiError> =>
  runApiRequest<Playlist>('createPlaylist', () => getApiClient().createPlaylist(data))

export const updatePlaylistEffect = (
  playlistId: string,
  updates: PlaylistUpdateData,
): Effect.Effect<Playlist, ApiError> =>
  runApiRequest<Playlist>('updatePlaylist', () => getApiClient().updatePlaylist(playlistId, updates))

export const deletePlaylistEffect = (playlistId: string): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('deletePlaylist', () => getApiClient().deletePlaylist(playlistId))

export const discordRpcClearActivityEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('discordRpcClearActivity', () => getApiClient().discordRpcClearActivity())

export const discordRpcIsRunningEffect = (): Effect.Effect<boolean, ApiError> =>
  runApiRequest<boolean>('discordRpcIsRunning', () => getApiClient().discordRpcIsRunning())

export const discordRpcSetActivityEffect = (
  activity: RpcActivity,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('discordRpcSetActivity', () =>
    getApiClient().discordRpcSetActivity(activity),
  )

export const discordRpcStartEffect = (appId: string): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('discordRpcStart', () => getApiClient().discordRpcStart(appId))

export const discordRpcStopEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('discordRpcStop', () => getApiClient().discordRpcStop())

export const audioSetAnalyzerEnabledEffect = (enabled: boolean): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioSetAnalyzerEnabled', () => getApiClient().audioSetAnalyzerEnabled(enabled))

export const audioAdvanceGaplessEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioAdvanceGapless', () => getApiClient().audioAdvanceGapless())

export const audioGetAllEqBandsEffect = (): Effect.Effect<number[], ApiError> =>
  runApiRequest<number[]>('audioGetAllEqBands', () => getApiClient().audioGetAllEqBands())

export const audioGetEqBandEffect = (band: number): Effect.Effect<number, ApiError> =>
  runApiRequest<number>('audioGetEqBand', () => getApiClient().audioGetEqBand(band))

export const audioGetPositionEffect = (): Effect.Effect<number, ApiError> =>
  runApiRequest<number>('audioGetPosition', () => getApiClient().audioGetPosition())

export const audioGetVolumeEffect = (): Effect.Effect<number, ApiError> =>
  runApiRequest<number>('audioGetVolume', () => getApiClient().audioGetVolume())

export const audioInitEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioInit', () => getApiClient().audioInit())

export const audioIsEqEnabledEffect = (): Effect.Effect<boolean, ApiError> =>
  runApiRequest<boolean>('audioIsEqEnabled', () => getApiClient().audioIsEqEnabled())

export const audioIsFinishedEffect = (): Effect.Effect<boolean, ApiError> =>
  runApiRequest<boolean>('audioIsFinished', () => getApiClient().audioIsFinished())

export const audioIsPlayingEffect = (): Effect.Effect<boolean, ApiError> =>
  runApiRequest<boolean>('audioIsPlaying', () => getApiClient().audioIsPlaying())

export const audioPauseEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioPause', () => getApiClient().audioPause())

export const audioPlayEffect = (url: string, token: string): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioPlay', () => getApiClient().audioPlay(url, token))

export const audioPrepareNextEffect = (url: string, token: string): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioPrepareNext', () => getApiClient().audioPrepareNext(url, token))

export const audioReinitializeEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioReinit', () => getApiClient().audioReinit())

export const audioResetEqEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioResetEq', () => getApiClient().audioResetEq())

export const audioResumeEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioResume', () => getApiClient().audioResume())

export const audioSeekEffect = (positionSecs: number): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioSeek', () => getApiClient().audioSeek(positionSecs))

export const audioSetEqBandEffect = (band: number, gainDb: number): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioSetEqBand', () => getApiClient().audioSetEqBand(band, gainDb))

export const audioSetEqEnabledEffect = (enabled: boolean): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioSetEqEnabled', () => getApiClient().audioSetEqEnabled(enabled))

export const audioSetVolumeEffect = (volume: number): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioSetVolume', () => getApiClient().audioSetVolume(volume))

export const audioStopEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('audioStop', () => getApiClient().audioStop())

export const mediaUpdateNowPlayingEffect = (payload: NowPlayingPayload): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('mediaUpdateNowPlaying', () => getApiClient().mediaUpdateNowPlaying(payload))

export const mediaSetPlaybackStatusEffect = (
  isPlaying: boolean,
  positionSecs?: number,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('mediaSetPlaybackStatus', () =>
    getApiClient().mediaSetPlaybackStatus(isPlaying, positionSecs),
  )

export const mediaSetButtonEnabledEffect = (
  button: string,
  enabled: boolean,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('mediaSetButtonEnabled', () => getApiClient().mediaSetButtonEnabled(button, enabled))

export const mediaClearNowPlayingEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('mediaClearNowPlaying', () => getApiClient().mediaClearNowPlaying())

export const lastFmAuthenticateEffect = (
  apiKey: string,
  apiSecret: string,
  token: string,
): Effect.Effect<LastFmCredentials, ApiError> =>
  runApiRequest<LastFmCredentials>('lastfmAuthenticate', () =>
    getApiClient().lastfmAuthenticate(apiKey, apiSecret, token),
  )

export const lastFmClearCredentialsEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('lastfmClearCredentials', () => getApiClient().lastfmClearCredentials())

export const lastFmStartAuthServerEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('lastfmStartAuthServer', () => getApiClient().lastfmStartAuthServer())

export const lastFmScrobbleEffect = (
  artist: string,
  track: string,
  album?: string,
  timestamp?: number,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('lastfmScrobble', () =>
    getApiClient().lastfmScrobble(artist, track, album, timestamp),
  )

export const lastFmSetCredentialsEffect = (
  credentials: LastFmCredentials,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('lastfmSetCredentials', () =>
    getApiClient().lastfmSetCredentials(credentials),
  )

export const lastFmUpdateNowPlayingEffect = (
  artist: string,
  track: string,
  album?: string,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('lastfmUpdateNowPlaying', () =>
    getApiClient().lastfmUpdateNowPlaying(artist, track, album),
  )

export const listenBrainzClearCredentialsEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('listenbrainzClearCredentials', () => getApiClient().listenbrainzClearCredentials())

export const listenBrainzPlayingNowEffect = (
  artist: string,
  track: string,
  album?: string,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('listenbrainzPlayingNow', () =>
    getApiClient().listenbrainzPlayingNow(artist, track, album),
  )

export const listenBrainzSetCredentialsEffect = (
  credentials: ListenBrainzCredentials,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('listenbrainzSetCredentials', () =>
    getApiClient().listenbrainzSetCredentials(credentials),
  )

export const listenBrainzSubmitListenEffect = (
  listen: ListenBrainzListen,
  timestamp: number,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('listenbrainzSubmitListen', () =>
    getApiClient().listenbrainzSubmitListen(listen, timestamp),
  )

export const listenBrainzValidateTokenEffect = (
  userToken: string,
): Effect.Effect<ListenBrainzCredentials, ApiError> =>
  runApiRequest<ListenBrainzCredentials>('listenbrainzValidateToken', () =>
    getApiClient().listenbrainzValidateToken(userToken),
  )

export const showMainWindowEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('showMainWindow', () => getApiClient().showMainWindow())

export const hideMainWindowEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('hideMainWindow', () => getApiClient().hideMainWindow())

export const quitApplicationEffect = (): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('quitApplication', () => getApiClient().quitApplication())

export const setMinimizeToTrayEffect = (minimizeToTray: boolean): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('setMinimizeToTray', () => getApiClient().setMinimizeToTray(minimizeToTray))

export const setCloseToTrayEffect = (closeToTray: boolean): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('setCloseToTray', () => getApiClient().setCloseToTray(closeToTray))

export const registerClientCapabilitiesEffect = (
  serverUrl: string,
  token: string,
  deviceId: string,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('registerClientCapabilities', () =>
    getApiClient().registerClientCapabilities(serverUrl, token, deviceId),
  )

export const reportPlaybackProgressEffect = (
  itemId: string,
  positionTicks: number,
  isPaused: boolean,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('reportPlaybackProgress', () =>
    getApiClient().reportPlaybackProgress(itemId, positionTicks, isPaused),
  )

export const reportPlaybackStartEffect = (
  itemId: string,
  positionTicks?: number,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('reportPlaybackStart', () =>
    getApiClient().reportPlaybackStart(itemId, positionTicks),
  )

export const reportPlaybackStopEffect = (
  itemId: string,
  positionTicks: number,
): Effect.Effect<void, ApiError> =>
  runApiRequest<void>('reportPlaybackStop', () =>
    getApiClient().reportPlaybackStop(itemId, positionTicks),
  )
