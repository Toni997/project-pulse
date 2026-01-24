import { create } from 'zustand'
import Track from '../types/Track'

interface TrackState {
  tracks: Track[]

  // actions
  addTrack: (track: Track) => void
  updateTrack: (index: number, updates: Partial<Track>) => void
  removeTrack: (index: number) => void
  setTracks: (tracks: Track[]) => void
}

export const useTrackStore = create<TrackState>((set) => ({
  tracks: [],

  addTrack: (track) =>
    set((state) => ({
      tracks: [...state.tracks, track],
    })),

  updateTrack: (index, updates) =>
    set((state) => ({
      tracks: state.tracks.map((track, i) =>
        i === index ? { ...track, ...updates } : track,
      ),
    })),

  removeTrack: (index) =>
    set((state) => ({
      tracks: state.tracks.filter((_, i) => i !== index),
    })),

  setTracks: (tracks) => ({ tracks }),
}))
