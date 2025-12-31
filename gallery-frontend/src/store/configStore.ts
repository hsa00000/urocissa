/**
 * These settings are used for both global application configuration (AppConfig)
 * and local UI preferences (e.g., isMobile, showFilenameChip).
 * Refactored to avoid dangerous default values for AppConfig and use inline types.
 */

import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'
import { AppConfig, getConfig, updateConfig } from '@/api/config'
import { tryWithMessageStore } from '@/script/utils/try_catch'

export const useConfigStore = (isolationId: IsolationId) =>
  defineStore('configStore' + isolationId, {
    state: (): {
      // AppConfig is nullable to indicate it hasn't been loaded yet
      config: AppConfig | undefined
      // UI State
      isMobile: boolean
      showFilenameChip: boolean
    } => ({
      config: undefined,
      isMobile: false,
      showFilenameChip: false
    }),
    getters: {
      disableImg: (state) => state.config?.disableImg ?? false
    },
    actions: {
      async fetchConfig() {
        // Return if already loaded to avoid redundant calls
        if (this.config) return

        return await tryWithMessageStore(isolationId, async () => {
          const data = await getConfig()
          this.config = data
        })
      },
      async updateConfig(newConfig: AppConfig & { oldPassword?: string }) {
        return await tryWithMessageStore(isolationId, async () => {
          await updateConfig(newConfig)
          this.config = newConfig
          return true
        })
      }
    }
  })()
