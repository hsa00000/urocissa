import { IsolationId, ScrollbarData } from '@type/types'
import { defineStore } from 'pinia'
export const useScrollbarStore = (isolationId: IsolationId) =>
  defineStore('scrollbarStore' + isolationId, {
    state: (): {
      scrollbarDataArray: ScrollbarData[]
      scrollbarDataArrayYear: ScrollbarData[]
      initialized: boolean
      isDragging: boolean
      isHovering: boolean
    } => ({
      scrollbarDataArray: [],
      scrollbarDataArrayYear: [],
      initialized: false,
      isDragging: false,
      isHovering: false
    }),
    actions: {
      initialize(scrollbarDataArray: ScrollbarData[]) {
        this.scrollbarDataArray = scrollbarDataArray
        this.scrollbarDataArrayYear = []
        let currentYear: number | null = null
        let lastDataForYear: ScrollbarData | null = null

        this.scrollbarDataArray.forEach((scrollbarData) => {
          if (currentYear !== scrollbarData.year) {
            if (lastDataForYear) {
              this.scrollbarDataArrayYear.push(lastDataForYear)
            }
            currentYear = scrollbarData.year
          }
          lastDataForYear = scrollbarData
        })

        // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
        if (lastDataForYear !== null) {
          this.scrollbarDataArrayYear.push(lastDataForYear)
        }


        this.initialized = true
      }
    }
  })()
