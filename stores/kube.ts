import { KubeConfig, NamedContext } from "@/app/kube/api";
import { create, StateCreator } from "zustand";
import { devtools, persist } from "zustand/middleware";

export interface KubeState {
  currentContext?: string;
  setCurrentContext?: (context: string) => void;
  kube?: KubeConfig;
  setKube: (kube?: KubeConfig) => void;
}

const initializer: StateCreator<KubeState> = (set) => ({
  currentContext: undefined,
  setCurrentContext: (context: string) => set({ currentContext: context }),
  kube: undefined,
  setKube: (kube?: KubeConfig) => set({ kube }),
});

function middleware(initializer: StateCreator<KubeState>) {
  return devtools(
    persist(initializer, {
      name: "kube-state",
    })
  );
}

export const useKubeStore = create<KubeState>()(middleware(initializer));
