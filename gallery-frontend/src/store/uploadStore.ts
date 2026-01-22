import { defineStore } from 'pinia'
import axios, { type AxiosProgressEvent } from 'axios'
import { useMessageStore } from './messageStore'
import { useModalStore } from './modalStore'
// import { useConstStore } from './constStore'
import { errorDisplay } from '@/script/utils/errorDisplay'
import { IsolationId, ProcessedImage } from '@type/types'
import { ProcessedImageSchema } from '@type/schemas'
import init, { process_image } from '@/wasm/gallery_wasm.js'
import { useConfigStore } from './configStore'

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
          try {
            if (files && files.length > 0) {
              await this.fileUpload([...files], albumId)
            }
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
        const messageStore = useMessageStore('mainId')
        const configStore = useConfigStore('mainId')

        this.status = 'Uploading'
        modalStore.showUploadModal = true

        const localMode = Boolean(configStore.config?.localMode) || false

        if (localMode) {
          console.log('local mode enable!')

          await this.localModeUpload(files, albumId)
          return
        }

        console.log('local mode not enable!')

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
              if (e.total !== undefined) {
                this.total = e.total
                // Axios types say loaded can be undefined
                if (typeof e.loaded === 'number') {
                  this.loaded = e.loaded
                }
                if (this.loaded !== undefined && this.total === this.loaded) {
                  this.status = 'Processing'
                }
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
          // Initialize WASM
          await init()

          for (const file of files) {
            const arrayBuffer = await file.arrayBuffer()
            const uint8Array = new Uint8Array(arrayBuffer)

            // Process image
            if (!file.type.startsWith('image/')) {
              console.warn(`Skipping WASM processing for non-image: ${file.name}`)
              continue
            }

            // Skip GIF
            if (file.type === 'image/gif') {
              continue
            }

            try {
              const processedRaw: unknown = process_image(uint8Array, file.name, file.lastModified)
              console.log('processedRaw', processedRaw)
              const processedResult = ProcessedImageSchema.safeParse(processedRaw)
              if (!processedResult.success) {
                console.error('Invalid WASM processed image payload', processedResult.error)
                continue
              }
              const processed: ProcessedImage = processedResult.data

              // Upload processed data
              const formData = new FormData()
              const nowMs = Date.now()

              // Backend expects ObjectSchema + ImageMetadata directly (camelCase)
              const payload = {
                object: {
                  id: processed.hash,
                  objType: 'image',
                  pending: false,
                  // Rust expects Option<Vec<u8>>; JSON must be a sequence (array), not a Uint8Array object.
                  thumbhash: processed.thumbhash ? Array.from(processed.thumbhash) : null,
                  description: null,
                  tags: [] as string[],
                  isFavorite: false,
                  isArchived: false,
                  isTrashed: false,
                  updateAt: nowMs
                },
                metadata: {
                  id: processed.hash,
                  size: processed.size,
                  width: processed.width,
                  height: processed.height,
                  ext: processed.extension,
                  phash: processed.phash ? Array.from(processed.phash) : null,
                  albums: albumId !== undefined ? [albumId] : [],
                  exifVec: processed.exif,
                  alias: [
                    {
                      file: file.name,
                      modified: file.lastModified,
                      scanTime: nowMs
                    }
                  ]
                }
              }

              formData.append('metadata', JSON.stringify(payload))

              // Append binary blobs
              // We send 'compressed' as the main optimized image.
              formData.append(
                'compressed',
                new Blob([processed.compressedImage], { type: 'image/jpeg' }),
                'compressed.jpg'
              )
              formData.append('original', file)

              const uploadUrl = `/upload-local`

              await axios.post(uploadUrl, formData, {
                headers: { 'Content-Type': 'multipart/form-data' }
              })
            } catch (e: unknown) {
              console.error('WASM processing failed for', file.name, e)
            }
          }
          this.status = 'Completed'
          messageStore.success('Files processed and uploaded locally')
        } catch (err) {
          this.status = 'Canceled'
          messageStore.error(errorDisplay(err))
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
