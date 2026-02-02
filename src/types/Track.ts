export interface BaseTrack {
  name: string
  volume: number // 0..1
  pan: number // -1..1
  muted: boolean
}

export interface SendAmount {
  busId: string
  amount: number // 0..1
}

export interface AudioTrack extends BaseTrack {
  id: string
  audioFile: string
  sends: SendAmount[]
}

export interface BusTrack extends BaseTrack {
  id: string
}

export interface MasterTrack extends BaseTrack {}
