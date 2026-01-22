// frontend/src/type/schemas.ts
import { z } from 'zod'
import { fixedBigRowHeight } from '@/type/constants'

export const AliasSchema = z.object({
  file: z.string(),
  modified: z.number(),
  scanTime: z.number()
})

export const displayElementSchema = z.object({
  displayWidth: z.number(),
  displayHeight: z.number(),
  displayTopPixelAccumulated: z.number().optional().default(0)
})

export const rowSchema = z.object({
  start: z.number(),
  end: z.number(),
  rowHeight: z.number().optional().default(fixedBigRowHeight),
  displayElements: z.array(displayElementSchema),
  topPixelAccumulated: z.number().default(0),
  rowIndex: z.number(),
  offset: z.number().optional().default(0)
})

export const rowWithOffsetSchema = z.object({
  row: rowSchema,
  offset: z.number(),
  windowWidth: z.number()
})

const BaseObjectRaw = z.object({
  id: z.string(),
  pending: z.boolean(),
  thumbhash: z.array(z.number()).nullable().optional().default(null),
  description: z.string().nullable().optional(),
  tags: z.array(z.string()).default([]),
  exifVec: z.record(z.string(), z.string()).default({}),
  isFavorite: z.boolean().default(false),
  isArchived: z.boolean().default(false),
  isTrashed: z.boolean().default(false),
  updateAt: z.number().default(0)
})

// 1. Image Schema
const ImageSchemaRaw = BaseObjectRaw.extend({
  type: z.literal('image'),
  width: z.number(),
  height: z.number(),
  ext: z.string(),
  size: z.number(),
  phash: z.array(z.number()).nullable().optional().default([]),
  albums: z.array(z.string()).default([]),
  alias: z.array(AliasSchema).default([])
}).transform((data) => ({
  type: 'image' as const,
  id: data.id,
  width: data.width,
  height: data.height,
  ext: data.ext,
  size: data.size,
  tags: data.tags,
  exif: data.exifVec,
  phash: data.phash,
  thumbhash: data.thumbhash,
  pending: data.pending,
  albums: data.albums,
  alias: data.alias,
  description: data.description,
  isFavorite: data.isFavorite,
  isArchived: data.isArchived,
  isTrashed: data.isTrashed,
  updateAt: data.updateAt
}))

// 2. Video Schema
const VideoSchemaRaw = BaseObjectRaw.extend({
  type: z.literal('video'),
  width: z.number(),
  height: z.number(),
  ext: z.string(),
  size: z.number(),
  duration: z.number().default(0),
  albums: z.array(z.string()).default([]),
  alias: z.array(AliasSchema).default([])
}).transform((data) => ({
  type: 'video' as const,
  id: data.id,
  width: data.width,
  height: data.height,
  ext: data.ext,
  size: data.size,
  duration: data.duration,
  tags: data.tags,
  exif: data.exifVec,
  thumbhash: data.thumbhash,
  pending: data.pending,
  albums: data.albums,
  alias: data.alias,
  description: data.description,
  isFavorite: data.isFavorite,
  isArchived: data.isArchived,
  isTrashed: data.isTrashed,
  updateAt: data.updateAt
}))

// 3. Album Schema
const AlbumSchemaRaw = BaseObjectRaw.extend({
  type: z.literal('album'),
  title: z.string().nullable(),
  startTime: z.number().nullable(),
  endTime: z.number().nullable(),
  lastModifiedTime: z.number(),
  cover: z.string().nullable(),
  itemCount: z.number(),
  itemSize: z.number(),
  shareList: z.record(z.string(), z.any()).default({})
}).transform((data) => ({
  type: 'album' as const,
  id: data.id,
  title: data.title,
  startTime: data.startTime,
  endTime: data.endTime,
  lastModifiedTime: data.lastModifiedTime,
  cover: data.cover,
  thumbhash: data.thumbhash,
  tags: data.tags,
  itemCount: data.itemCount,
  itemSize: data.itemSize,
  pending: data.pending,
  description: data.description,
  isFavorite: data.isFavorite,
  isArchived: data.isArchived,
  isTrashed: data.isTrashed,
  updateAt: data.updateAt,
  shareList: data.shareList
}))

export const BackendDataParser = z.union([ImageSchemaRaw, VideoSchemaRaw, AlbumSchemaRaw])

export const prefetchSchema = z.object({
  timestamp: z.number(),
  dataLength: z.number(),
  locateTo: z.number().nullable()
})

export const ShareSchema = z.object({
  url: z.string().max(64),
  description: z.string(),
  password: z.string().nullable(),
  showMetadata: z.boolean(),
  showDownload: z.boolean(),
  showUpload: z.boolean(),
  exp: z.number()
})

export const ResolvedShareSchema = z.object({
  share: ShareSchema,
  albumId: z.string().max(64),
  albumTitle: z.string().nullable()
})

export const prefetchReturnSchema = z
  .object({
    prefetch: prefetchSchema,
    token: z.string(),
    resolvedShareOpt: ResolvedShareSchema.nullable()
  })
  .transform((data) => ({
    prefetch: data.prefetch,
    token: data.token,
    resolvedShare: data.resolvedShareOpt
  }))

export const scrollbarDataSchema = z.object({
  index: z.number(),
  year: z.number(),
  month: z.number()
})

export const tagInfoSchema = z.object({
  tag: z.string(),
  number: z.number()
})

export const albumInfoSchema = z
  .object({
    albumId: z.string(),
    albumName: z.string().nullable(),
    shareList: z.record(z.string(), ShareSchema)
  })
  .transform((albumData) => ({
    albumId: albumData.albumId,
    albumName: albumData.albumName,
    shareList: new Map(Object.entries(albumData.shareList)),
    displayName: albumData.albumName ?? 'Untitled'
  }))

export const databaseTimestampSchema = z.object({
  abstractData: BackendDataParser,
  timestamp: z.number(),
  token: z.string()
})

export const SubRowSchema = z.object({
  displayElements: z.array(displayElementSchema)
})

const Uint8ArrayLikeSchema = z.union([
  z.instanceof(Uint8Array),
  z.instanceof(ArrayBuffer),
  z.array(z.number())
])

const ExifValueSchema = z.union([z.string(), z.number(), z.boolean(), z.bigint(), z.null()])

const ExifSchema = z
  .preprocess((value) => {
    if (value instanceof Map) {
      return Object.fromEntries(Array.from(value.entries(), ([k, v]) => [String(k), v]))
    }
    return value
  }, z.union([z.record(z.string(), ExifValueSchema), z.array(z.unknown()), z.string()]))
  .optional()
  .nullable()

export const ProcessedImageSchema = z
  .object({
    hash: z.string(),
    width: z.number(),
    height: z.number(),
    size: z.number(),
    extension: z.string(),
    thumbhash: Uint8ArrayLikeSchema.nullable(),
    phash: Uint8ArrayLikeSchema.nullable().optional(),
    exif: ExifSchema,
    compressedImage: Uint8ArrayLikeSchema,
    lastModified: z.number()
  })
  .transform((data) => ({
    ...data,
    thumbhash: data.thumbhash ? new Uint8Array(data.thumbhash) : null,
    phash: data.phash ? new Uint8Array(data.phash) : null,
    compressedImage: new Uint8Array(data.compressedImage),
    exif:
      data.exif !== null &&
      data.exif !== undefined &&
      typeof data.exif === 'object' &&
      !Array.isArray(data.exif)
        ? Object.fromEntries(
            Object.entries(data.exif).map(([key, value]) => [key, String(value)])
          )
        : {}
  }))

export const PublicConfigSchema = z.object({
  address: z.string(),
  port: z.number(),
  limits: z.record(z.string(), z.string()), // HashMap<String, String>
  syncPaths: z.array(z.string()), // HashSet<PathBuf> deserializes to array
  discordHookUrl: z.string().nullable().optional(),
  readOnlyMode: z.boolean(),
  disableImg: z.boolean()
})



export const TokenResponseSchema = z.object({
  token: z.string()
})

export const serverErrorSchema = z.object({
  kind: z.string().optional(),
  message: z.string().optional(),
  status: z.string().optional(),
  context: z.array(z.string()).optional(),
  error: z.string().optional(),
  chain: z.array(z.string()).optional()
})
