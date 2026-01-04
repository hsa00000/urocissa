import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export const useEditStore = (isolationId: IsolationId) =>
  defineStore('editStore' + isolationId, {
    state: (): {
      processingRotate: Set<string>
      processingRegenerate: Set<string>
    } => ({
      processingRotate: new Set(),
      processingRegenerate: new Set()
    }),
    actions: {
      addRotate(hash: string) {
        this.processingRotate.add(hash)
      },
      removeRotate(hash: string) {
        this.processingRotate.delete(hash)
      },
      hasRotate(hash: string) {
        return this.processingRotate.has(hash)
      },
      addRegenerate(hash: string) {
        this.processingRegenerate.add(hash)
      },
      removeRegenerate(hash: string) {
        this.processingRegenerate.delete(hash)
      },
      hasRegenerate(hash: string) {
        return this.processingRegenerate.has(hash)
      }
    }
  })()
