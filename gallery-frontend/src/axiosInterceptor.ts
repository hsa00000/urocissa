import axios, { AxiosInstance, AxiosError, InternalAxiosRequestConfig } from 'axios'
import { errorDisplay } from '@/script/utils/errorDisplay'
import { useShareStore } from '@/store/shareStore'
import { useModalStore } from '@/store/modalStore'
import { useMessageStore } from '@/store/messageStore'
import { useRedirectionStore } from '@/store/redirectionStore'

// Variable to track the last 401 error time for debouncing
let last401Timestamp = 0

export function setupMainAxiosInterceptor() {
  // Request interceptor
  axios.interceptors.request.use((config: InternalAxiosRequestConfig) => {
    const shareStore = useShareStore('mainId')

    if (typeof shareStore.albumId === 'string' && typeof shareStore.shareId === 'string') {
      config.headers.set('x-album-id', shareStore.albumId)
      config.headers.set('x-share-id', shareStore.shareId)

      // Add password header if it exists
      if (shareStore.password) {
        config.headers.set('x-share-password', shareStore.password)
      }
    }

    return config
  })

  // Response interceptor
  axios.interceptors.response.use(
    (response) => {
      return response
    },
    async (error: AxiosError) => {
      if (error.response) {
        const status = error.response.status
        const modalStore = useModalStore('mainId')
        const messageStore = useMessageStore('mainId')
        const shareStore = useShareStore('mainId')
        const redirectionStore = useRedirectionStore('mainId')

        const isSharePage = shareStore.albumId && shareStore.shareId

        if (isSharePage) {
          if (status === 401) {
            // Check for stale requests (zombies)
            const sentPassword = error.config?.headers?.['x-share-password']
            const currentPassword = shareStore.password

            // If we have a password now, but the request didn't send it (or sent a different one),
            // it means this request is stale. Do not trigger the modal again.
            if (currentPassword && sentPassword !== currentPassword) {
              return Promise.reject(error)
            }

            // 401: Password required
            if (!modalStore.showShareLoginModal) {
              shareStore.isLinkExpired = false
              modalStore.showShareLoginModal = true
            }
          } else if (status === 403) {
            // 403: Link expired or access denied
            const displayMsg = errorDisplay(error)
            messageStore.error(
              displayMsg !== 'Unknown error occurred'
                ? displayMsg
                : 'Share link has expired or access is denied.'
            )

            if (!modalStore.showShareLoginModal) {
              shareStore.isLinkExpired = true
              modalStore.showShareLoginModal = true
            }
          }
        } else {
          if (status === 401) {
            // Debounce 401 handling to prevent duplicate modals/snackbars
            const now = Date.now()
            if (now - last401Timestamp < 1000) {
              return Promise.reject(error)
            }
            last401Timestamp = now

            messageStore.error('Session expired or unauthorized. Please login.')
            await redirectionStore.redirectionToLogin()
          } else if (status === 403) {
            const displayMsg = errorDisplay(error)
            messageStore.error(
              displayMsg !== 'Unknown error occurred' ? displayMsg : 'Access denied.'
            )
          } else if (status === 405) {
            messageStore.error('Read only mode is on.')
          } else {
            // Generic error handler for other Axios errors (e.g. 404, 400, etc.)
            // This ensures no silent failures now that tryWithMessageStore ignores Axios errors
            const displayMsg = errorDisplay(error)
            messageStore.error(displayMsg)
          }
        }
      }

      return Promise.reject(error)
    }
  )
}

export function setupWorkerAxiosInterceptor(
  axiosInstance: AxiosInstance,
  notify: (payload: { text: string; color: 'error' }) => void
) {
  axiosInstance.interceptors.response.use(
    (response) => response,
    (error: AxiosError) => {
      // Handle server errors (500) or other unexpected errors
      // Use the unified errorDisplay function to parse the error
      if (error.response?.status === 500) {
        const errorMessage = errorDisplay(error)
        notify({ text: errorMessage, color: 'error' })
      }
      return Promise.reject(error)
    }
  )
}
