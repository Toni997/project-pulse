import { Clip } from './Clip'
import Id from './Id'

export enum TrackKind {
  Audio = 'Audio',
  Sampler = 'Sampler',
  Instrument = 'Instrument',
  Bus = 'Bus',
  Master = 'Master',
}

export interface BaseTrack {
  name: string
  volume: number // 0..1
  pan: number // -1..1
  muted: boolean
  kind: TrackKind
  clips: Record<Id, Clip>
}

export interface SendAmount {
  busId: string
  amount: number // 0..1
}

export interface AudioTrack extends BaseTrack {
  id: string
}

export interface SamplerTrack extends BaseTrack {
  id: string
  sourceId: string
}

export type GeneratorTrack = AudioTrack | SamplerTrack

export interface BusTrack extends BaseTrack {
  id: string
}

export interface MasterTrack extends BaseTrack {}

export type GeneratorOrBusTrack = GeneratorTrack | BusTrack
