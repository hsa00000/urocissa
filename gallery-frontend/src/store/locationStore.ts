import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export const useLocationStore = (isolationId: IsolationId) =>
  defineStore('locateStore' + isolationId, {
    state: (): {
      /**
       * Index of the first photo that appears (partially) in the viewport
       */
      locationIndex: number
      anchor: number | null
      /**
       * Global item index for a pending two-step locate jump.
       * Set during the first (row-level) jump; consumed by fetchRowReturn
       * to refine the scroll to the exact subrow position.
       */
      pendingLocateTarget: number | null
      highlightedIndex: number | null
    } => ({
      locationIndex: 0,
      anchor: null,
      pendingLocateTarget: null,
      highlightedIndex: null
    }),
    actions: {
      clearAll() {
        this.locationIndex = 0
        this.anchor = null
        this.pendingLocateTarget = null
        this.highlightedIndex = null
      },
      triggerForResize() {
        this.anchor = null
      }
    }
  })()
