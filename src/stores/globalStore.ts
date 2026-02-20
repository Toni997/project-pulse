import { create } from 'zustand'

interface GlobalState {
  isGlobalLoading: boolean
  setGlobalLoading: (isGlobalLoading: boolean) => void
}

export const useGlobalStore = create<GlobalState>((set) => ({
  isGlobalLoading: false,
  setGlobalLoading: (isGlobalLoading) => set({ isGlobalLoading }),
}))
