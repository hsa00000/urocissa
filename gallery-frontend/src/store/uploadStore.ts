import { defineStore } from 'pinia'
import axios, { type AxiosProgressEvent } from 'axios'
import { useMessageStore } from './messageStore'
import { useModalStore } from './modalStore'
import { errorDisplay } from '@/script/utils/errorDisplay'
import { IsolationId } from '@type/types'
import init, { process_image } from '@/wasm/gallery_wasm.js'
import { useConfigStore } from './configStore'

interface ProcessImageForUploadResult {
  metadataJson: string
  compressedJpeg: Uint8Array
  hash: string
}

function toUint8Array(value: unknown): Uint8Array | null {
  if (value instanceof Uint8Array) return value
  if (value instanceof ArrayBuffer) return new Uint8Array(value)
  if (Array.isArray(value) && value.every((v) => typeof v === 'number')) {
    return new Uint8Array(value)
  }
  return null
}

function toArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  // Blob typings can be picky about ArrayBuffer vs SharedArrayBuffer; normalize via copy.
  if (bytes.buffer instanceof ArrayBuffer) {
    return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength)
  }
  const arrayBuffer = new ArrayBuffer(bytes.byteLength)
  new Uint8Array(arrayBuffer).set(bytes)
  return arrayBuffer
}

function toProcessImageForUploadResult(value: unknown): ProcessImageForUploadResult | null {
  if (typeof value !== 'object' || value === null) return null

  const record = value as Record<string, unknown>
  if (typeof record.metadataJson !== 'string') return null
  if (typeof record.hash !== 'string') return null

  const compressedJpeg = toUint8Array(record.compressedJpeg)
  if (compressedJpeg === null) return null

  return {
    metadataJson: record.metadataJson,
    compressedJpeg,
    hash: record.hash
  }
}

export const useUploadStore = (isolationId: IsolationId) =>
  defineStore('uploadStore' + isolationId, {
    state: () => ({
      status: 'Canceled' as 'Uploading' | 'Processing' | 'Canceled' | 'Completed',
      total: undefined as number | undefined,
      loaded: undefined as number | undefined,
      startTime: undefined as number | undefined,
      abortController: null as AbortController | null
    }),

    getters: {
      percentComplete: (state): number =>
        state.total !== undefined && state.loaded !== undefined && state.total > 0
          ? Math.floor((state.loaded / state.total) * 100)
          : 0,

      elapsedTime: (state): number =>
        state.startTime !== undefined ? (Date.now() - state.startTime) / 1000 : 0,

      uploadSpeed(): number {
        const elapsed = this.elapsedTime
        return elapsed > 0 && this.loaded !== undefined ? this.loaded / elapsed : 0 // bytes/sec
      },

      remainingTime(): number {
        const speed = this.uploadSpeed
        if (speed > 0 && this.total !== undefined && this.loaded !== undefined) {
          return (this.total - this.loaded) / speed // seconds
        }
        return 0
      }
    },

    actions: {
      triggerFileInput(albumId: string | undefined): void {
        const fileInput = document.createElement('input')
        fileInput.type = 'file'
        fileInput.multiple = true
        fileInput.style.display = 'none'

        const handleChange = async (event: Event): Promise<void> => {
          const target = event.target as HTMLInputElement
          const files = target.files
          if (!files || files.length === 0) {
            document.body.removeChild(fileInput)
            return
          }
          try {
            await this.fileUpload([...files], albumId)
          } finally {
            document.body.removeChild(fileInput)
          }
        }

        // Wrapper to satisfy no-misused-promises
        const changeHandler = (e: Event): void => {
          void handleChange(e)
        }

        fileInput.addEventListener('change', changeHandler, { once: true })
        document.body.appendChild(fileInput)
        fileInput.click()
      },

      async fileUpload(files: File[], albumId: string | undefined): Promise<void> {
        const modalStore = useModalStore('mainId')
        const configStore = useConfigStore('mainId')

        this.status = 'Uploading'
        modalStore.showUploadModal = true

        const localMode = configStore.config?.localMode ?? false

        if (localMode) {
          await this.localModeUpload(files, albumId)
          return
        }

        await this.remoteUpload(files, albumId)
      },

      async remoteUpload(files: File[], albumId: string | undefined): Promise<void> {
        const messageStore = useMessageStore('mainId')

        const formData = new FormData()
        for (const file of files) {
          formData.append('file', file)
          formData.append('lastModified', String(file.lastModified))
        }

        const uploadUrl =
          albumId !== undefined
            ? `/upload?presigned_album_id_opt=${encodeURIComponent(albumId)}`
            : `/upload`

        const abortController = new AbortController()
        this.abortController = abortController
        this.total = this.loaded = 0
        this.startTime = Date.now()

        try {
          await axios.post(uploadUrl, formData, {
            headers: { 'Content-Type': 'multipart/form-data' },
            signal: abortController.signal,
            onUploadProgress: (e: AxiosProgressEvent) => {
              if (e.total === undefined) return
              this.total = e.total
              if (typeof e.loaded !== 'number') return
              this.loaded = e.loaded
              if (this.loaded === this.total) {
                this.status = 'Processing'
              }
            }
          })

          this.status = 'Completed'
          messageStore.success('Files uploaded successfully')
        } catch (err) {
          this.status = 'Canceled'
          messageStore.error(errorDisplay(err))
        }
      },

      async localModeUpload(files: File[], albumId: string | undefined): Promise<void> {
        const messageStore = useMessageStore('mainId')
        this.status = 'Processing'

        try {
          await init()
          for (const file of files) {
            await this.processAndUploadFile(file, albumId)
          }
          this.status = 'Completed'
          messageStore.success('Files processed and uploaded locally')
        } catch (err) {
          this.status = 'Canceled'
          messageStore.error(errorDisplay(err))
        }
      },

      async processAndUploadFile(file: File, albumId: string | undefined): Promise<void> {
        if (!file.type.startsWith('image/')) {
          console.warn(`Skipping WASM processing for non-image: ${file.name}`)
          return
        }

        if (file.type === 'image/gif') {
          return
        }

        const arrayBuffer = await file.arrayBuffer()
        const uint8Array = new Uint8Array(arrayBuffer)

        let processed
        try {
          processed = toProcessImageForUploadResult(
            process_image(uint8Array, file.name, file.lastModified, albumId)
          )
        } catch (e: unknown) {
          console.error('WASM processing failed for', file.name, e)
          return
        }

        if (processed === null) {
          return
        }

        const formData = new FormData()
        formData.append('metadata', processed.metadataJson)
        formData.append(
          'compressed',
          new Blob([toArrayBuffer(processed.compressedJpeg)], { type: 'image/jpeg' }),
          'compressed.jpg'
        )
        formData.append('original', file)

        const uploadUrl = `/upload-local`

        try {
          await axios.post(uploadUrl, formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
          })
        } catch (e: unknown) {
          console.error('Upload failed for', file.name, e)
        }
      },

      cancelUpload(): void {
        if (this.abortController) {
          this.abortController.abort()
          this.status = 'Canceled'
        }
      }
    }
  })()
