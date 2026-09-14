import { create } from 'zustand'
import {
  DEFAULT_BARS_VISIBLE,
  DEFAULT_PX_PER_BEAT,
  DEFAULT_TRACK_SCALE,
} from '../helpers/constants'

interface TimelineState {
  playheadBeats: number
  pxPerBeat: number
  trackScale: number
  barsVisible: number
  scrollBeats: number
  scrollTracks: number

  setPlayheadBeats: (playheadBeats: number) => void
  setPxPerBeat: (pxPerBeat: number) => void
  setTrackScale: (trackScale: number) => void
  setBarsVisible: (barsVisible: number) => void
  setScrollBeats: (scrollBeats: number) => void
  setScrollTracks: (scrollTracks: number) => void
}

export const useTimelineStore = create<TimelineState>((set) => ({
  playheadBeats: 0,
  pxPerBeat: DEFAULT_PX_PER_BEAT,
  trackScale: DEFAULT_TRACK_SCALE,
  barsVisible: DEFAULT_BARS_VISIBLE,
  scrollBeats: 0,
  scrollTracks: 0,

  setPlayheadBeats: (playheadBeats) => set({ playheadBeats }),
  setPxPerBeat: (pxPerBeat) => set({ pxPerBeat }),
  setTrackScale: (trackScale) => set({ trackScale }),
  setBarsVisible: (barsVisible) => set({ barsVisible }),
  setScrollBeats: (scrollBeats) => set({ scrollBeats }),
  setScrollTracks: (scrollTracks) => set({ scrollTracks }),
}))
