import {create, StateCreator} from 'zustand'
import {devtools, persist} from 'zustand/middleware'

export interface DockState {
  progress: number | null
  badge: string | null
}

export interface AppState {
  isDarkMode: boolean
  title?: string
  isSidebarCollapsed: boolean
  isCollapsible: boolean
  dock: DockState
  setIsSidebarCollapsed: (isSidebarCollapsed: boolean) => void
  setIsDarkMode: (isDarkMode: boolean) => void
  setTitle: (title: string) => void
  toggleDarkMode: () => void
  setDockState: (dock: DockState) => void
}

const initializer: StateCreator<AppState> = (set) => ({
  isDarkMode: false,
  isSidebarCollapsed: true,
  isCollapsible: false,
  dock: {
    progress: null,
    badge: null,
  },
  setTitle: (title: string) => set({title}),
  setIsSidebarCollapsed: (isSidebarCollapsed: boolean) => set({isSidebarCollapsed}),
  setIsDarkMode: (isDarkMode: boolean) => set({isDarkMode}),
  toggleDarkMode: () => set(s => ({isDarkMode: !s.isDarkMode})),
  setDockState: (dock: DockState) => set({dock}),
})

function middleware(initializer: StateCreator<AppState>) {
  return devtools(
    persist(initializer, {
      name: 'app-state',
    }),
  )
}

export const useAppStore = create<AppState>()(middleware(initializer))