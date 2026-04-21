export interface Clip {
  id: string
  trackId: string
  name: string
  sourceOffsetSamples: number
  startPpq: number
  lengthPpq: number
  sourceId: string
}

export interface ClipToInsert {
  startPpq: number
  trackId: string | null
  sourcePath: string
}
