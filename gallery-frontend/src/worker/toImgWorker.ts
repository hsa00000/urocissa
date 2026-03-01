import { readAndCompressImage } from '@misskey-dev/browser-image-resizer'
import { bindActionDispatch, createHandler } from 'typesafe-agent-events'
import { fromImgWorker, toImgWorker } from '@/worker/workerApi'
import {
  ProcessAbortPayload,
  ProcessImagePayload,
  ProcessSmallImagePayload
} from '@/worker/workerApi'
import axiosRetry from 'axios-retry'
import axios from 'axios'
import { getSrc } from '@utils/getter'
import { setupWorkerAxiosInterceptor } from './workerAxiosInterceptor'

const postToMainImg = bindActionDispatch(fromImgWorker, self.postMessage.bind(self))
const controllerMap = new Map<number, AbortController>()
const workerAxios = axios.create()

axiosRetry(workerAxios, {
  retries: 0,
  retryDelay: () => 200,
  retryCondition: (error) => {
    if (axios.isCancel(error)) return false
    const response = error.response
    return response ? response.status !== 200 : true
  }
})

setupWorkerAxiosInterceptor(workerAxios, postToMainImg.notification)

// ================== Blob Cache ==================

const IMG_CACHE_NAME = 'img-blob-cache-v1'

/**
 * In-memory cache of fetched image Blobs, keyed by image hash.
 *
 * On resize, the main thread clears imgStore.imgUrl and re-queues visible
 * images. Without this cache every image would repeat the full network fetch.
 * With this cache the worker skips straight to createImageBitmap → resize,
 * which is sub-millisecond for typical thumbnails. The cache lives for the
 * lifetime of the worker (terminated on component unmount via
 * workerStore.terminateWorker()).
 */
const blobCache = new Map<string, Blob>()

function imgCacheKey(hash: string): string {
  return `https://img-blob-cache.internal/${hash}`
}

async function getFromDiskCache(hash: string): Promise<Blob | undefined> {
  if (typeof caches === 'undefined') return undefined
  try {
    const cache = await caches.open(IMG_CACHE_NAME)
    const response = await cache.match(imgCacheKey(hash))
    if (response) {
      const blob = await response.blob()
      if (blob.size > 0) return blob
      await cache.delete(imgCacheKey(hash))
    }
  } catch {
    // Cache API unavailable
  }
  return undefined
}

async function putToDiskCache(hash: string, blob: Blob): Promise<void> {
  if (typeof caches === 'undefined') return
  try {
    const cache = await caches.open(IMG_CACHE_NAME)
    await cache.put(imgCacheKey(hash), new Response(blob))
  } catch {
    // Non-critical
  }
}

async function purgeFromCaches(hash: string): Promise<void> {
  blobCache.delete(hash)
  if (typeof caches === 'undefined') return
  try {
    const cache = await caches.open(IMG_CACHE_NAME)
    await cache.delete(imgCacheKey(hash))
  } catch {
    // Non-critical
  }
}

// ================== Handlers ==================

const handler = createHandler<typeof toImgWorker>({
  async processSmallImage(event: ProcessSmallImagePayload) {
    try {
      const controller = new AbortController()
      controllerMap.set(event.index, controller)

      // Layer 1: In-memory cache
      let blob = blobCache.get(event.hash)

      // Layer 2: Cache API (disk-persistent)
      if (blob === undefined) {
        blob = await getFromDiskCache(event.hash)
        if (blob !== undefined) {
          blobCache.set(event.hash, blob)
        }
      }

      // Layer 3: Network fetch
      if (blob === undefined) {
        const headers: Record<string, string> = {}
        if (event.albumId !== null) headers['x-album-id'] = event.albumId
        if (event.shareId !== null) headers['x-share-id'] = event.shareId
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        if (event.password) headers['x-share-password'] = event.password

        headers.Authorization = `Bearer ${event.hashToken}`

        const config = {
          signal: controller.signal,
          responseType: 'blob' as const,
          headers,
          timestampToken: event.timestampToken
        }

        const response = await workerAxios.get<Blob>(
          getSrc(event.hash, false, 'jpg', event.updatedAt),
          config
        )

        blob = response.data
        blobCache.set(event.hash, blob)
        void putToDiskCache(event.hash, blob)
      }

      controllerMap.delete(event.index)
      const img = await createImageBitmap(blob)

      const albumMode = event.albumMode === true
      const converted: Blob = await readAndCompressImage(img, {
        argorithm: 'bilinear',
        quality: 1,
        maxWidth: albumMode
          ? img.width *
            Math.max(event.width / img.width, event.height / img.height) *
            event.devicePixelRatio
          : event.width * event.devicePixelRatio,
        maxHeight: albumMode
          ? img.height *
            Math.max(event.width / img.width, event.height / img.height) *
            event.devicePixelRatio
          : event.height * event.devicePixelRatio
      })

      const objectUrl = URL.createObjectURL(converted)
      postToMainImg.smallImageProcessed({ index: event.index, url: objectUrl })
    } catch (error) {
      if (axios.isCancel(error)) return
      // Purge potentially corrupt blob from caches so the next attempt re-fetches
      void purgeFromCaches(event.hash)
      console.error(error)
    }
  },

  async processImage(event: ProcessImagePayload) {
    try {
      // Layer 1: In-memory cache
      let blob = blobCache.get(event.hash)

      // Layer 2: Cache API (disk-persistent)
      if (blob === undefined) {
        blob = await getFromDiskCache(event.hash)
        if (blob !== undefined) {
          blobCache.set(event.hash, blob)
        }
      }

      // Layer 3: Network fetch
      if (blob === undefined) {
        const headers: Record<string, string> = {}
        if (event.albumId !== null) headers['x-album-id'] = event.albumId
        if (event.shareId !== null) headers['x-share-id'] = event.shareId
        // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
        if (event.password) headers['x-share-password'] = event.password
        headers.Authorization = `Bearer ${event.hashToken}`

        const config = {
          responseType: 'blob' as const,
          headers,
          timestampToken: event.timestampToken
        }

        const response = await workerAxios.get<Blob>(
          getSrc(event.hash, false, 'jpg', event.updatedAt),
          config
        )

        blob = response.data
        blobCache.set(event.hash, blob)
        void putToDiskCache(event.hash, blob)
      }

      const img = await createImageBitmap(blob)

      const offscreenCanvas = new OffscreenCanvas(img.width, img.height)
      const context = offscreenCanvas.getContext('2d')
      context?.drawImage(img, 0, 0)

      const orientedImgBlob = await offscreenCanvas.convertToBlob()
      const objectUrl = URL.createObjectURL(orientedImgBlob)

      postToMainImg.imageProcessed({ index: event.index, url: objectUrl })
    } catch (error) {
      // Purge potentially corrupt blob from caches so the next attempt re-fetches
      void purgeFromCaches(event.hash)
      console.error(error)
    }
  },

  processAbort(event: ProcessAbortPayload) {
    const controller = controllerMap.get(event.index)
    if (controller !== undefined) {
      controller.abort()
      controllerMap.delete(event.index)
    }
  }
})

self.addEventListener('message', (e) => {
  handler(e.data as ReturnType<(typeof toImgWorker)[keyof typeof toImgWorker]>)
})
