import { create } from 'zustand'
import {
  BEATS_PER_BAR,
  DEFAULT_BARS_VISIBLE,
  DEFAULT_PX_PER_BEAT,
} from '../helpers/constants'

interface TimelineState {
  playheadPosition: number
  stageWidth: number
  stageHeight: number
  pxPerBeat: number
  barsVisible: number

  setPlayheadPosition: (playheadPosition: number) => void
  setStageWidth: (stageWidth: number) => void
  setStageHeight: (stageHeight: number) => void
  setPxPerBeat: (pxPerBeat: number) => void
  setBarsVisible: (barsVisible: number) => void
}

export const useTimelineStore = create<TimelineState>((set) => ({
  playheadPosition: 0,
  stageWidth: DEFAULT_BARS_VISIBLE * DEFAULT_PX_PER_BEAT * BEATS_PER_BAR,
  stageHeight: window.innerHeight,
  pxPerBeat: DEFAULT_PX_PER_BEAT,
  barsVisible: DEFAULT_BARS_VISIBLE,

  setPlayheadPosition: (playheadPosition) => set({ playheadPosition }),
  setStageWidth: (stageWidth) => set({ stageWidth }),
  setStageHeight: (stageHeight) => set({ stageHeight }),
  setPxPerBeat: (pxPerBeat) => set({ pxPerBeat }),
  setBarsVisible: (barsVisible) => set({ barsVisible }),
}))
