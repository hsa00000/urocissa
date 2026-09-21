import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  clearOwnedObjectUrls,
  deleteOwnedObjectUrl,
  setOwnedObjectUrl
} from './objectUrlRegistry'

let revokeObjectUrl: ReturnType<typeof vi.spyOn>

beforeEach(() => {
  revokeObjectUrl = vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => undefined)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('objectUrlRegistry', () => {
  it('releases a replaced URL and the final retained URL on clear', () => {
    const urls = new Map<number, string>()

    setOwnedObjectUrl(urls, 1, 'blob:first')
    setOwnedObjectUrl(urls, 1, 'blob:second')

    expect(revokeObjectUrl).toHaveBeenCalledOnce()
    expect(revokeObjectUrl).toHaveBeenCalledWith('blob:first')

    clearOwnedObjectUrls(urls)
    expect(revokeObjectUrl).toHaveBeenNthCalledWith(2, 'blob:second')
  })

  it('keeps a shared URL alive until its final owner releases it', () => {
    const directUrls = new Map<number, string>()
    const collectionUrls = new Map<number, string>()

    setOwnedObjectUrl(directUrls, 0, 'blob:shared')
    setOwnedObjectUrl(collectionUrls, 7, 'blob:shared')
    deleteOwnedObjectUrl(directUrls, 0)

    expect(revokeObjectUrl).not.toHaveBeenCalled()

    deleteOwnedObjectUrl(collectionUrls, 7)
    expect(revokeObjectUrl).toHaveBeenCalledOnce()
    expect(revokeObjectUrl).toHaveBeenCalledWith('blob:shared')
  })

  it('does not retain the same URL twice for the same slot', () => {
    const urls = new Map<number, string>()

    setOwnedObjectUrl(urls, 1, 'blob:stable')
    setOwnedObjectUrl(urls, 1, 'blob:stable')
    deleteOwnedObjectUrl(urls, 1)

    expect(revokeObjectUrl).toHaveBeenCalledOnce()
  })
})
