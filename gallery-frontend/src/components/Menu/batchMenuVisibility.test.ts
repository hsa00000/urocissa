import { describe, expect, it } from 'vitest'
import type { RouteMeta } from 'vue-router'
import { shouldShowBatchEditAlbums } from './batchMenuVisibility'

type BatchMenuRouteMeta = Pick<RouteMeta, 'baseName' | 'level'>

function routeMeta(baseName: RouteMeta['baseName'], level: number): BatchMenuRouteMeta {
  return { baseName, level }
}

describe('shouldShowBatchEditAlbums', () => {
  it.each([1, 2])('hides the action for album-card selections at route level %i', (level) => {
    expect(shouldShowBatchEditAlbums(routeMeta('albums', level))).toBe(false)
  })

  it.each([3, 4])('shows the action inside an album at route level %i', (level) => {
    expect(shouldShowBatchEditAlbums(routeMeta('albums', level))).toBe(true)
  })

  it('keeps the action available for media selections outside the albums route', () => {
    expect(shouldShowBatchEditAlbums(routeMeta('all', 1))).toBe(true)
  })
})
