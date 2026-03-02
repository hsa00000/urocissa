import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export const useScrollTopStore = (isolationId: IsolationId) =>
  defineStore('scrollTopStore' + isolationId, {
    state: (): {
      scrollTop: number
      useCompensation: boolean
    } => ({
      scrollTop: 0,
      useCompensation: false
    }),
    actions: {}
  })()
