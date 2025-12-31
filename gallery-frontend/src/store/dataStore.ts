import type { EnrichedUnifiedData, IsolationId } from '@type/types'
import { defineStore } from 'pinia'

export const useDataStore = (isolationId: IsolationId) =>
  defineStore('DataStore' + isolationId, {
    state: (): {
      data: Map<number, EnrichedUnifiedData> // dataIndex -> data
      hashMapData: Map<string, number> // hash -> dataIndex
      batchFetched: Map<number, boolean> // Tracks the batches of image metadata that have been fetched
    } => ({
      data: new Map(),
      hashMapData: new Map(),
      batchFetched: new Map()
    }),
    actions: {
      // Should be cleared when the layout is changed
      clearAll() {
        this.data.clear()
        this.hashMapData.clear()
        this.batchFetched.clear()
      },
      addTags(index: number, tags: string[]): boolean {
        const data = this.data.get(index)
        if (!data) {
          // Index does not exist
          return false
        }

        tags.forEach((tag) => {
          if (!data.tags.includes(tag)) {
            data.tags.push(tag)
          }
        })
        return true
      },
      removeTags(index: number, tags: string[]): boolean {
        const data = this.data.get(index)
        if (!data) {
          return false
        }

        data.tags = data.tags.filter((tag) => !tags.includes(tag))
        return true
      },
      addAlbums(index: number, albums: string[]): boolean {
        const data = this.data.get(index)
        if (!data) {
          return false
        }

        // Albums are only applicable to Images/Videos, not nested Albums
        if (data.type === 'image' || data.type === 'video') {
          albums.forEach((album) => {
            if (!data.albums.includes(album)) {
              data.albums.push(album)
            }
          })
          return true
        }

        return false
      },
      removeAlbums(index: number, albums: string[]): boolean {
        const data = this.data.get(index)
        if (!data) {
          return false
        }

        if (data.type === 'image' || data.type === 'video') {
          data.albums = data.albums.filter((album) => !albums.includes(album))
          return true
        }

        return false
      }
    }
  })()
