// This file initializes the Vue 3 application, sets up the router, state management (Pinia), and Vuetify UI framework.
// It is referenced in index.html to bootstrap the app, configures the application with a dark theme, and mounts it to the DOM.

// Importing core dependencies and main component
import { createApp } from 'vue'
import App from '@/components/App.vue'

// Importing router and state management
import router from '@/route/routes'
import { createPinia } from 'pinia'

// Importing global styles and icons
import '@/style/common.scss'
import '@mdi/font/css/materialdesignicons.css'

// Importing Vuetify UI framework and configuration
import { createVuetify } from 'vuetify'
import * as components from 'vuetify/components'
import * as directives from 'vuetify/directives'
import axios, { AxiosError, InternalAxiosRequestConfig } from 'axios'
import { useRedirectionStore } from '@/store/redirectionStore'
import { useShareStore } from '@/store/shareStore'
import { useConstStore } from '@/store/constStore'
import { useModalStore } from '@/store/modalStore'
import { useMessageStore } from '@/store/messageStore'
import { HandledError } from '@/type/types'

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
          messageStore.error('Share link has expired or access is denied.')
          if (!modalStore.showShareLoginModal) {
            shareStore.isLinkExpired = true
            modalStore.showShareLoginModal = true
          }
        }
      } else {
        if (status === 401) {
          messageStore.error('Session expired or unauthorized. Please login.')
          await redirectionStore.redirectionToLogin()

          const handledError: HandledError = error
          handledError.isHandled = true
        } else if (status === 403) {
          messageStore.error('Access denied.')
          const handledError: HandledError = error
          handledError.isHandled = true
        } else if (status === 405) {
          messageStore.error('Read only mode is on.')
          const handledError: HandledError = error
          handledError.isHandled = true
        }
      }
    }

    return Promise.reject(error)
  }
)
// Create Vue application instance
const app = createApp(App)

// Setup state management (Pinia) early so stores can be used outside components
const pinia = createPinia()
app.use(pinia)

// Ensure const store is available and load theme preference before creating Vuetify
const constStore = useConstStore('mainId')
await constStore.loadTheme()

// Configure Vuetify and set default theme (use Vuetify's built-in theme palettes)
const vuetify = createVuetify({
  components,
  directives,
  theme: {
    // 'light' | 'dark' | 'system'
    defaultTheme: constStore.theme === 'light' ? 'light' : 'dark'
  }
})

// Apply necessary plugins and mount the app
app.use(router)
app.use(vuetify)
app.mount('#app')
