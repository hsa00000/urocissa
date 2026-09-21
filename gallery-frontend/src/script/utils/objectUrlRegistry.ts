const objectUrlReferenceCounts = new Map<string, number>()

export function retainObjectUrl(url: string): void {
  objectUrlReferenceCounts.set(url, (objectUrlReferenceCounts.get(url) ?? 0) + 1)
}

export function releaseObjectUrl(url: string): void {
  const referenceCount = objectUrlReferenceCounts.get(url)
  if (referenceCount !== undefined && referenceCount > 1) {
    objectUrlReferenceCounts.set(url, referenceCount - 1)
    return
  }

  objectUrlReferenceCounts.delete(url)
  URL.revokeObjectURL(url)
}

export function setOwnedObjectUrl<Key>(
  target: Map<Key, string>,
  key: Key,
  url: string
): void {
  const previousUrl = target.get(key)
  if (previousUrl === url) return

  retainObjectUrl(url)
  target.set(key, url)
  if (previousUrl !== undefined) releaseObjectUrl(previousUrl)
}

export function deleteOwnedObjectUrl<Key>(target: Map<Key, string>, key: Key): void {
  const url = target.get(key)
  if (url === undefined) return

  target.delete(key)
  releaseObjectUrl(url)
}

export function clearOwnedObjectUrls<Key>(target: Map<Key, string>): void {
  const urls = [...target.values()]
  target.clear()
  urls.forEach(releaseObjectUrl)
}
