import {create, StateCreator} from 'zustand'
import {devtools, persist} from 'zustand/middleware'

export interface AppState {
  isDarkMode: boolean
  title?: string
  isSidebarCollapsed: boolean
  isCollapsible: boolean
  setIsSidebarCollapsed: (isSidebarCollapsed: boolean) => void
  setIsDarkMode: (isDarkMode: boolean) => void
  setTitle: (title: string) => void
  toggleDarkMode: () => void
}

const initializer: StateCreator<AppState> = (set) => ({
  isDarkMode: false,
  isSidebarCollapsed: true,
  isCollapsible: false,
  setTitle: (title: string) => set({title}),
  setIsSidebarCollapsed: (isSidebarCollapsed: boolean) => set({isSidebarCollapsed}),
  setIsDarkMode: (isDarkMode: boolean) => set({isDarkMode}),
  toggleDarkMode: () => set(s => ({isDarkMode: !s.isDarkMode})),
})

function middleware(initializer: StateCreator<AppState>) {
  return devtools(
    persist(initializer, {
      name: 'app-state',
    }),
  )
}

export const useAppStore = create<AppState>()(middleware(initializer))