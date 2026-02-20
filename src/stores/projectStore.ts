import { create } from 'zustand'
import { AudioTrack, BusTrack, MasterTrack } from '../types/Track'

export interface ProjectState {
  ppq: number
  tempo_bpm: number
  time_signature: [Number, number] | null
  master: MasterTrack
  tracks: AudioTrack[]
  buses: BusTrack[]

  addAudioTrack: (track: AudioTrack) => void
  addBusTrack: (track: BusTrack) => void

  updateMasterTrack: (updates: Partial<MasterTrack>) => void
  updateAudioTrack: (id: string, updates: Partial<AudioTrack>) => void
  updateBusTrack: (id: string, updates: Partial<BusTrack>) => void
  // updateSend: (trackId: string, send: SendAmount) => void

  removeAudioTrack: (id: string) => void
  removeBusTrack: (id: string) => void
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
  },
  tracks: [],
  buses: [],

  addAudioTrack: (track) =>
    set((state) => ({
      tracks: [...state.tracks, track],
    })),

  addBusTrack: (track) =>
    set((state) => ({
      buses: [...state.buses, track],
    })),

  updateMasterTrack: (updates) =>
    set((state) => ({
      master: { ...state.master, ...updates },
    })),

  updateAudioTrack: (id, updates) =>
    set((state) => ({
      tracks: state.tracks.map((t) => (t.id === id ? { ...t, ...updates } : t)),
    })),

  updateBusTrack: (id, updates) =>
    set((state) => ({
      buses: state.buses.map((b) => (b.id === id ? { ...b, ...updates } : b)),
    })),

  // updateSend: (trackId, send) =>
  //   set((state) => ({
  //     tracks: state.tracks.map((t) =>
  //       t.id === trackId
  //         ? {
  //             ...t,
  //             sends: t.sends.some((s) => s.busId === send.busId)
  //               ? t.sends.map((s) => (s.busId === send.busId ? send : s))
  //               : [...t.sends, send],
  //           }
  //         : t,
  //     ),
  //   })),

  removeAudioTrack: (id) =>
    set((state) => ({
      tracks: state.tracks.filter((t) => t.id !== id),
    })),

  removeBusTrack: (id) =>
    set((state) => ({
      buses: state.buses.filter((b) => b.id !== id),
    })),
}))
