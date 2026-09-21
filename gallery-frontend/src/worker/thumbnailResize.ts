export type ThumbnailFitMode = 'contain' | 'cover'

export interface ThumbnailSize {
  width: number
  height: number
}

export interface ThumbnailRenderPlan {
  sourceX: number
  sourceY: number
  sourceWidth: number
  sourceHeight: number
  outputWidth: number
  outputHeight: number
}

function assertPositiveDimensions(...dimensions: number[]): void {
  if (dimensions.some((dimension) => !Number.isFinite(dimension) || dimension <= 0)) {
    throw new Error('thumbnail dimensions must be positive finite numbers')
  }
}

export function calculateThumbnailSize(
  sourceWidth: number,
  sourceHeight: number,
  maxWidth: number,
  maxHeight: number
): ThumbnailSize {
  assertPositiveDimensions(sourceWidth, sourceHeight, maxWidth, maxHeight)

  const aspectRatio = sourceWidth / sourceHeight
  const outputWidth = Math.min(sourceWidth, maxWidth, aspectRatio * maxHeight)
  const width = Math.max(1, Math.floor(outputWidth))
  const height = Math.max(
    1,
    Math.floor(Math.min(sourceHeight * (width / sourceWidth), maxHeight))
  )
  return { width, height }
}

export function calculateThumbnailRenderPlan(
  sourceWidth: number,
  sourceHeight: number,
  displayWidth: number,
  displayHeight: number,
  devicePixelRatio: number,
  fitMode: ThumbnailFitMode
): ThumbnailRenderPlan {
  assertPositiveDimensions(
    sourceWidth,
    sourceHeight,
    displayWidth,
    displayHeight,
    devicePixelRatio
  )

  const targetWidth = Math.max(1, Math.ceil(displayWidth * devicePixelRatio))
  const targetHeight = Math.max(1, Math.ceil(displayHeight * devicePixelRatio))

  if (fitMode === 'contain') {
    const output = calculateThumbnailSize(
      sourceWidth,
      sourceHeight,
      targetWidth,
      targetHeight
    )
    return {
      sourceX: 0,
      sourceY: 0,
      sourceWidth,
      sourceHeight,
      outputWidth: output.width,
      outputHeight: output.height
    }
  }

  const targetAspectRatio = targetWidth / targetHeight
  const sourceAspectRatio = sourceWidth / sourceHeight
  let croppedWidth = sourceWidth
  let croppedHeight = sourceHeight

  if (sourceAspectRatio > targetAspectRatio) {
    croppedWidth = sourceHeight * targetAspectRatio
  } else if (sourceAspectRatio < targetAspectRatio) {
    croppedHeight = sourceWidth / targetAspectRatio
  }

  // Avoid manufacturing pixels when a source is smaller than its display box.
  // CSS may still enlarge the result, but the worker does not spend memory or
  // encoding time on an upscaled intermediate bitmap.
  const outputScale = Math.min(
    1,
    croppedWidth / targetWidth,
    croppedHeight / targetHeight
  )

  return {
    sourceX: (sourceWidth - croppedWidth) / 2,
    sourceY: (sourceHeight - croppedHeight) / 2,
    sourceWidth: croppedWidth,
    sourceHeight: croppedHeight,
    outputWidth: Math.max(1, Math.floor(targetWidth * outputScale)),
    outputHeight: Math.max(1, Math.floor(targetHeight * outputScale))
  }
}

export async function resizeThumbnailBlob(
  blob: Blob,
  displayWidth: number,
  displayHeight: number,
  devicePixelRatio: number,
  fitMode: ThumbnailFitMode
): Promise<Blob> {
  const image = await createImageBitmap(blob)
  try {
    const plan = calculateThumbnailRenderPlan(
      image.width,
      image.height,
      displayWidth,
      displayHeight,
      devicePixelRatio,
      fitMode
    )

    // Backend thumbnails are already JPEG. If no crop or resize is needed,
    // retain the original blob and avoid a redundant encode cycle.
    if (
      plan.sourceX === 0 &&
      plan.sourceY === 0 &&
      plan.sourceWidth === image.width &&
      plan.sourceHeight === image.height &&
      plan.outputWidth === image.width &&
      plan.outputHeight === image.height &&
      blob.type === 'image/jpeg'
    ) {
      return blob
    }

    // Render directly into the final DPR-sized surface. This avoids the old
    // full-source canvas and ImageData copy while matching CSS object-fit.
    const canvas = new OffscreenCanvas(plan.outputWidth, plan.outputHeight)
    const context = canvas.getContext('2d')
    if (context === null) {
      throw new Error('thumbnail resize canvas context is unavailable')
    }

    context.imageSmoothingEnabled = true
    context.imageSmoothingQuality = 'high'
    context.drawImage(
      image,
      plan.sourceX,
      plan.sourceY,
      plan.sourceWidth,
      plan.sourceHeight,
      0,
      0,
      plan.outputWidth,
      plan.outputHeight
    )
    return await canvas.convertToBlob({ type: 'image/jpeg', quality: 1 })
  } finally {
    image.close()
  }
}
