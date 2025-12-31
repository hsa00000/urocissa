import { thumbHashToDataURL } from 'thumbhash'
import { UnifiedData } from '@type/types'

/**
 * Enriches data with a thumbhash URL.
 * Backend data is already flattened by Zod transformation.
 */
export function enrichWithThumbhash<T extends UnifiedData>(
  data: T
): T & { thumbhashUrl: string | null } {
  const thumbhashUrl = data.thumbhash ? thumbHashToDataURL(data.thumbhash) : null
  return { ...data, thumbhashUrl }
}

/**
 * Returns the appropriate filename/title for display.
 */
export function getFilename(data: UnifiedData): string {
  if (data.type === 'image' || data.type === 'video') {
    return data.alias[0]?.file.split('/').pop() ?? ''
  }
  return data.title ?? ''
}
