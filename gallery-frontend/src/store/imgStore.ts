import { IsolationId } from '@type/types'
import { defineStore } from 'pinia'
import { clearThumbnailElements } from '@/script/utils/thumbnailElementRegistry'
import {
  clearOwnedObjectUrls,
  deleteOwnedObjectUrl,
  setOwnedObjectUrl
} from '@/script/utils/objectUrlRegistry'
import { markRaw } from 'vue'

export const useImgStore = (isolationId: IsolationId) =>
  defineStore('imgStore' + isolationId, {
    state: (): {
      imgUrl: Map<number, string> // dataIndex -> blobUrl
      imgOriginal: Map<number, string> // dataIndex -> blobUrl
    } => ({
      // Thumbnail DOM updates are handled by the element registry. Keeping this
      // cache raw avoids notifying Vue for every worker completion in a burst.
      imgUrl: markRaw(new Map()),
      imgOriginal: new Map()
    }),
    actions: {
      setThumbnailUrl(index: number, url: string) {
        setOwnedObjectUrl(this.imgUrl, index, url)
      },
      setOriginalUrl(index: number, url: string) {
        setOwnedObjectUrl(this.imgOriginal, index, url)
      },
      deleteThumbnailUrl(index: number) {
        deleteOwnedObjectUrl(this.imgUrl, index)
      },
      deleteOriginalUrl(index: number) {
        deleteOwnedObjectUrl(this.imgOriginal, index)
      },
      // Should be cleared when the layout is changed
      clearAll() {
        clearThumbnailElements(isolationId)
        clearOwnedObjectUrls(this.imgUrl)
        clearOwnedObjectUrls(this.imgOriginal)
      },
      clearForResize() {
        clearThumbnailElements(isolationId)
        clearOwnedObjectUrls(this.imgUrl)
      }
    }
  })()
