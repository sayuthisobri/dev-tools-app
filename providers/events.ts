import {EventCallback, listen as listenTauri} from '@tauri-apps/api/event'
import {isTauri} from '@/lib/utils'

export async function listen<T>(event: string, callback: EventCallback<T>, mode: 'tauri' | 'window' = 'tauri') {

  if (mode === 'tauri' && isTauri()) {
    return await listenTauri<T>(event, callback)
  } else {
    window.addEventListener(event, (e) => {
      callback(e as any)
    })
  }

} 