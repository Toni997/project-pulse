import { create } from 'zustand'

interface DragState {
  audioPath: string | null
  setAudioPath: (audioPath: string | null) => void
  clear: () => void
}

export const useDragStore = create<DragState>((set) => ({
  audioPath: null,
  setAudioPath: (audioPath) => set({ audioPath }),
  clear: () => set({ audioPath: null }),
}))

