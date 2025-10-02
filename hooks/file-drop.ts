import { useEffect } from 'react'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { isWebMode } from '@/lib/utils'

export const useFileDrop = () => {
  useEffect(() => {
    if (isWebMode()) return
    let deRegister: () => void
    getCurrentWebviewWindow().onDragDropEvent(e => {
      if (e.payload.type == 'drop') {
        console.log('on drop', e, document.elementFromPoint(e.payload.position.x, e.payload.position.y))
      }
    })
      .then(fn => deRegister = fn)

    return () => {
      if (typeof deRegister == 'function') deRegister()
    }
  })
}

