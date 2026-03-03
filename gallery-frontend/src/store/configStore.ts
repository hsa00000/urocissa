import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'
import { AppConfig, getConfig, updateConfig } from '@/api/config'
import { tryWithMessageStore } from '@/script/utils/try_catch'

export const useConfigStore = (isolationId: IsolationId) =>
  defineStore('configStore' + isolationId, {
    state: (): {
      config: AppConfig | undefined
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
        if (this.config) return

        return await tryWithMessageStore(isolationId, async () => {
          const data = await getConfig()
          this.config = data
        })
      },
      async updateConfig(newConfig: Partial<AppConfig>) {
        return await tryWithMessageStore(isolationId, async () => {
          await updateConfig(newConfig)
          if (this.config) {
            Object.assign(this.config, newConfig)
          } else {
            const data = await getConfig()
            this.config = data
          }
          return true
        })
      }
    }
  })()
