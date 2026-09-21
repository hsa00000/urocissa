import type { RouteMeta } from 'vue-router'

type BatchMenuRouteMeta = Pick<RouteMeta, 'baseName' | 'level'>

/**
 * Album cards are selected on the first two albums route levels. From level 3 onward,
 * the selection belongs to the media collection inside an album and can be moved
 * between albums like media selected on any other gallery page.
 */
export function shouldShowBatchEditAlbums({ baseName, level }: BatchMenuRouteMeta): boolean {
  return baseName !== 'albums' || level >= 3
}
