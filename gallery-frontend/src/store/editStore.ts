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
        // Get the current promise chain for this hash, or start a new one
        const previousTask = this.rotationQueue.get(hash) || Promise.resolve()

        // Chain the new task to run after the previous one completes
        const newTask = previousTask
          .then(() => task())
          .catch((error) => {
            console.error(`Rotation task failed for hash ${hash}:`, error)
          })

        // Update the queue with the new tail of the chain
        this.rotationQueue.set(hash, newTask)

        // Wait for this specific task to finish (optional, depending on if caller needs to await)
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
        const count = this.rotationCounts.get(hash) || 0
        this.rotationCounts.set(hash, count + 1)
      }
    }
  })()
