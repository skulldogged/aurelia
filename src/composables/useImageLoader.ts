export const useImageLoader = () => {
  // Get cached image or fetch it
  const getImageUrl = (
    itemId: string,
    serverUrl: string,
    token: string,
    imageType: string = 'Primary',
  ): string | null => {
    if (!itemId || !serverUrl || !token) return null

    const baseUrl = `${serverUrl.replace(/\/$/, '')}/Items/${itemId}/Images/${imageType}`
    return `${baseUrl}?api_key=${token}`
  }

  return {
    getImageUrl,
  }
}
