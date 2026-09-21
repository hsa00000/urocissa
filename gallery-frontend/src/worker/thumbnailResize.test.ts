import { afterEach, describe, expect, it, vi } from 'vitest'
import {
  calculateThumbnailRenderPlan,
  calculateThumbnailSize,
  resizeThumbnailBlob
} from './thumbnailResize'

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('calculateThumbnailSize', () => {
  it('fits landscape and portrait images inside the requested display box', () => {
    expect(calculateThumbnailSize(4000, 3000, 400, 300)).toEqual({
      width: 400,
      height: 300
    })
    expect(calculateThumbnailSize(3000, 4000, 400, 300)).toEqual({
      width: 225,
      height: 300
    })
  })

  it('does not upscale an image that already fits', () => {
    expect(calculateThumbnailSize(100, 50, 400, 300)).toEqual({
      width: 100,
      height: 50
    })
  })
})

describe('calculateThumbnailRenderPlan', () => {
  it('center-crops the reported tall thumbnail to the restricted-ratio tile', () => {
    expect(calculateThumbnailRenderPlan(263, 1920, 125, 250, 1, 'cover')).toEqual({
      sourceX: 0,
      sourceY: 697,
      sourceWidth: 263,
      sourceHeight: 526,
      outputWidth: 125,
      outputHeight: 250
    })
  })

  it('uses DPR-sized output without changing the cover crop', () => {
    expect(calculateThumbnailRenderPlan(263, 1920, 125, 250, 2, 'cover')).toEqual({
      sourceX: 0,
      sourceY: 697,
      sourceWidth: 263,
      sourceHeight: 526,
      outputWidth: 250,
      outputHeight: 500
    })
  })

  it('center-crops wide sources and retains contain mode', () => {
    expect(calculateThumbnailRenderPlan(1920, 263, 250, 125, 1, 'cover')).toEqual({
      sourceX: 697,
      sourceY: 0,
      sourceWidth: 526,
      sourceHeight: 263,
      outputWidth: 250,
      outputHeight: 125
    })
    expect(calculateThumbnailRenderPlan(3000, 4000, 400, 300, 1, 'contain')).toEqual({
      sourceX: 0,
      sourceY: 0,
      sourceWidth: 3000,
      sourceHeight: 4000,
      outputWidth: 225,
      outputHeight: 300
    })
  })

  it('does not create an upscaled output for a small cover source', () => {
    expect(calculateThumbnailRenderPlan(100, 50, 500, 250, 1, 'cover')).toEqual({
      sourceX: 0,
      sourceY: 0,
      sourceWidth: 100,
      sourceHeight: 50,
      outputWidth: 100,
      outputHeight: 50
    })
  })

  it('rejects invalid source, target, or DPR dimensions', () => {
    expect(() => calculateThumbnailRenderPlan(0, 100, 50, 50, 1, 'cover')).toThrow(
      'thumbnail dimensions must be positive finite numbers'
    )
    expect(() => calculateThumbnailRenderPlan(100, 100, -1, 50, 1, 'cover')).toThrow(
      'thumbnail dimensions must be positive finite numbers'
    )
    expect(() => calculateThumbnailRenderPlan(100, 100, 50, 50, 0, 'cover')).toThrow(
      'thumbnail dimensions must be positive finite numbers'
    )
  })
})

describe('resizeThumbnailBlob', () => {
  it('renders the cover crop directly into the final canvas', async () => {
    const close = vi.fn()
    const image = { width: 263, height: 1920, close }
    const drawImage = vi.fn()
    const context = {
      imageSmoothingEnabled: false,
      imageSmoothingQuality: 'low',
      drawImage
    }
    const output = new Blob(['resized'], { type: 'image/jpeg' })
    const convertToBlob = vi.fn(
      (options: ImageEncodeOptions): Promise<Blob> => {
        void options
        return Promise.resolve(output)
      }
    )
    const canvasSizes: [number, number][] = []

    class TestOffscreenCanvas {
      constructor(width: number, height: number) {
        canvasSizes.push([width, height])
      }

      getContext() {
        return context
      }

      convertToBlob(options: ImageEncodeOptions): Promise<Blob> {
        return convertToBlob(options)
      }
    }

    vi.stubGlobal('createImageBitmap', vi.fn().mockResolvedValue(image))
    vi.stubGlobal('OffscreenCanvas', TestOffscreenCanvas)

    const result = await resizeThumbnailBlob(
      new Blob(['source'], { type: 'image/jpeg' }),
      125,
      250,
      1,
      'cover'
    )

    expect(result).toBe(output)
    expect(canvasSizes).toEqual([[125, 250]])
    expect(context.imageSmoothingEnabled).toBe(true)
    expect(context.imageSmoothingQuality).toBe('high')
    expect(drawImage).toHaveBeenCalledWith(image, 0, 697, 263, 526, 0, 0, 125, 250)
    expect(convertToBlob).toHaveBeenCalledWith({ type: 'image/jpeg', quality: 1 })
    expect(close).toHaveBeenCalledOnce()
  })
})
