import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export const useEditStore = (isolationId: IsolationId) =>
  defineStore('editStore' + isolationId, {
    state: (): {
      processingRegenerate: Set<string>
      rotationCounts: Map<string, number>
      rotationQueue: Map<string, Promise<void>>
    } => ({
      processingRegenerate: new Set(),
      rotationCounts: new Map(),
      rotationQueue: new Map()
    }),
    actions: {
      async queueRotate(hash: string, task: () => Promise<void>) {
        const previousTask = this.rotationQueue.get(hash) ?? Promise.resolve()

        const newTask = previousTask
          .then(() => task())
          .catch((error: unknown) => {
            console.error(`Rotation task failed for hash ${hash}:`, error)
          })

        this.rotationQueue.set(hash, newTask)
        await newTask
      },
      addRegenerate(hash: string) {
        this.processingRegenerate.add(hash)
      },
      removeRegenerate(hash: string) {
        this.processingRegenerate.delete(hash)
      },
      hasRegenerate(hash: string) {
        return this.processingRegenerate.has(hash)
      },
      incrementRotation(hash: string) {
        const count = this.rotationCounts.get(hash) ?? 0
        this.rotationCounts.set(hash, count + 1)
      }

    }
  })()
