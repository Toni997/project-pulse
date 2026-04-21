import { create } from 'zustand'
import Id from '../types/Id'
import {
  GeneratorOrBusTrack,
  MasterTrack,
  TrackKind,
} from '../types/Track'
import type { Clip } from '../types/Clip'

export interface ProjectState {
  ppq: number
  tempo_bpm: number
  time_signature: [Number, number] | null
  tracks: Record<Id, GeneratorOrBusTrack>
  master: MasterTrack
  generatorTracksOrder: Id[]
  busesOrder: Id[]

  addTrack: (track: GeneratorOrBusTrack) => void

  updateMasterTrack: (updates: Partial<MasterTrack>) => void
  updateTrack: (id: Id, updates: Partial<GeneratorOrBusTrack>) => void

  removeTrack: (id: Id) => void

  addClip: (clip: Clip) => void
  updateClip: (clip: Clip) => void
}

export const useProjectStore = create<ProjectState>((set) => ({
  ppq: 960,
  tempo_bpm: 128.0,
  time_signature: [4, 4],
  master: {
    name: 'Master',
    volume: 1.0,
    pan: 0,
    muted: false,
    clips: {},
    kind: TrackKind.Master,
  },
  tracks: {},
  generatorTracksOrder: [],
  busesOrder: [],

  addTrack: (track) =>
    set((state) => {
      const kind = track.kind
      const nextTracks = { ...state.tracks, [track.id]: track }

      if (kind === TrackKind.Bus) {
        const nextBusesOrder = state.busesOrder.includes(track.id)
          ? state.busesOrder
          : [...state.busesOrder, track.id]
        return { tracks: nextTracks, busesOrder: nextBusesOrder }
      }

      const nextGeneratorTracksOrder = state.generatorTracksOrder.includes(
        track.id,
      )
        ? state.generatorTracksOrder
        : [...state.generatorTracksOrder, track.id]

      return { tracks: nextTracks, generatorTracksOrder: nextGeneratorTracksOrder }
    }),

  updateMasterTrack: (updates) =>
    set((state) => ({
      master: { ...state.master, ...updates },
    })),

  updateTrack: (id, updates) =>
    set((state) => {
      const existing = state.tracks[id]
      if (!existing) return {}

      const next = { ...existing, ...updates } as GeneratorOrBusTrack
      const prevKind = existing.kind
      const nextKind = next.kind

      const nextState: Partial<ProjectState> = {
        tracks: { ...state.tracks, [id]: next },
      }

      if (prevKind !== nextKind) {
        if (prevKind === TrackKind.Bus) {
          nextState.busesOrder = state.busesOrder.filter((tid) => tid !== id)
          nextState.generatorTracksOrder = state.generatorTracksOrder.includes(id)
            ? state.generatorTracksOrder
            : [...state.generatorTracksOrder, id]
        } else if (nextKind === TrackKind.Bus) {
          nextState.generatorTracksOrder = state.generatorTracksOrder.filter(
            (tid) => tid !== id,
          )
          nextState.busesOrder = state.busesOrder.includes(id)
            ? state.busesOrder
            : [...state.busesOrder, id]
        }
      }

      return nextState
    }),

  removeTrack: (id) =>
    set((state) => {
      if (!state.tracks[id]) return {}
      const { [id]: _, ...rest } = state.tracks
      return {
        tracks: rest,
        generatorTracksOrder: state.generatorTracksOrder.filter((tid) => tid !== id),
        busesOrder: state.busesOrder.filter((tid) => tid !== id),
      }
    }),

  addClip: (clip) =>
    set((state) => {
      const trackId = clip.trackId
      const existing = state.tracks[trackId]
      if (!existing) return {}

      return {
        tracks: {
          ...state.tracks,
          [trackId]: {
            ...existing,
            clips: { ...existing.clips, [clip.id]: clip },
          },
        },
      }
    }),

  updateClip: (clip) =>
    set((state) => {
      const trackId = clip.trackId
      const existing = state.tracks[trackId]
      if (!existing) return {}
      if (!existing.clips[clip.id]) return {}

      return {
        tracks: {
          ...state.tracks,
          [trackId]: {
            ...existing,
            clips: { ...existing.clips, [clip.id]: clip },
          },
        },
      }
    }),
}))
