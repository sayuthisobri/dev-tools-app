import {create, StateCreator} from 'zustand';
import {devtools, persist} from 'zustand/middleware';

export interface FilesPageState {
  isNewDialogOpen: boolean
  setNewDialogOpen: (open: boolean) => void
}

const initializer: StateCreator<FilesPageState> = (set) => ({
  isNewDialogOpen: true,
  setNewDialogOpen: (open: boolean) => set({isNewDialogOpen: open}),
});

function middleware(initializer: StateCreator<FilesPageState>) {
  return devtools(
    persist(initializer, {
      name: 'file-page-state',
    }),
  );
}

export const useFilesPageStore = create<FilesPageState>()(middleware(initializer));
