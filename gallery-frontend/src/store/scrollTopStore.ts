import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export type ScrollMode = 'nativeTop' | 'compensation' | 'nativeBottom'

export const useScrollTopStore = (isolationId: IsolationId) =>
  defineStore('scrollTopStore' + isolationId, {
    state: (): {
      scrollTop: number
      scrollMode: ScrollMode
    } => ({
      scrollTop: 0,
      scrollMode: 'nativeTop'
    }),
    actions: {}
  })()
