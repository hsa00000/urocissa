import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export type ScrollMode = 'nativeTop' | 'compensation' | 'nativeBottom'

export const useScrollTopStore = (isolationId: IsolationId) =>
  defineStore('scrollTopStore' + isolationId, {
    state: (): {
      scrollTop: number
      scrollMode: ScrollMode
      /**
       * True after clearForResize resets totalHeight to an estimate.
       * While settling, the nativeTop scroll handler guards against browser
       * clamping of DOM scrollTop (caused by totalHeight shrinking as real
       * row heights replace estimates). Cleared once virtual ≤ DOM,
       * meaning the DOM can fully represent the virtual position.
       */
      settling: boolean
    } => ({
      scrollTop: 0,
      scrollMode: 'nativeTop',
      settling: false
    }),
    actions: {}
  })()
