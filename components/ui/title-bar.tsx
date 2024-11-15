'use client'
import React, {MouseEventHandler} from 'react'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {isTauri} from '@tauri-apps/api/core'
import {useAppStore} from '@/stores'
// import { isTauri } from '@/lib/utils'

export default function TitleBar({children}: { children?: React.ReactNode }): React.ReactNode {
  const {title} = useAppStore()
  const onMouseDown: MouseEventHandler = (e) => {
    const appWindow = isTauri() ? getCurrentWindow() : undefined
    console.log(e.detail, e.buttons)
    if (e.buttons === 1) {
      // Primary (left) button
      e.detail === 2
        ? appWindow?.toggleMaximize() // Maximize on double click
        : appWindow?.startDragging() // Else start dragging
    }
  }

  return (
    <div className={`fixed h-7 w-full bg-gray-500 bg-opacity-35 top-0 z-[55] shadow overflow-hidden
    flex items-center justify-between
    `}
         onMouseDown={onMouseDown}>
      <div className={'flex-none'}></div>
      <div className={'pointer-events-none text-md flex justify-center absolute left-1/2 transform -translate-x-1/2'}>
        {title!! && (<h1 className={'text-gray-900/70 text-sm'}>{title}</h1>)}
      </div>
      <div className={'pointer-events-none flex-none'}></div>
      {children}
    </div>
  )
}
