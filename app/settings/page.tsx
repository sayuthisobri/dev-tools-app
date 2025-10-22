'use client'
import React from 'react'
import {Button} from '@/components/ui/button'
import {toast} from 'sonner'
import {useAppStore} from '@/stores'
import {PageTitle} from '@/components/ui/typography'
import {invoke} from '@tauri-apps/api/core'

export default function SettingPage(): React.ReactNode {
  const {toggleDarkMode, setTitle} = useAppStore()
  return (
    <div className={'m-2 flex flex-col gap-2'}>
      <PageTitle>Settings</PageTitle>
      <div className="flex gap-2">
        <Button
          variant={'primary'}
          onClick={() => {
            toast.info('Info', {
              action: <Button variant={'primary'} onClick={() => {
                console.log('nice')
              }}>Action</Button>,
              cancel: <Button variant={'destructive'} onClick={() => {
                console.log('opss')
              }}>Nope</Button>,
              duration: 50000,
              closeButton: true,
            })
          }}>Info</Button>
        <Button
          variant={'success'}
          whileTap={{scale: 0.9}}
          onClick={() => {
            toast.success('Success', {
              description: 'Congratulations! You have successfully completed the task.',
              duration: 50000,
              closeButton: true,
            })
          }}>Success</Button>
        <Button
          variant={'warning'}
          whileTap={{scale: 0.9}}
          onClick={() => {
            toast.warning('Warning', {
              description: 'This is a warning',
              duration: 50000,
              closeButton: true,
            })
          }}>Warning</Button>
        <Button
          variant={'error'}
          onClick={() => {
            toast.error('Opps something went wrong', {
              duration: 50000,
              closeButton: true,
            })
          }}>Error</Button>
        <Button
          onClick={() => {
            toast.promise(new Promise((resolve) => setTimeout(resolve, 3000)), {
              loading: 'Loading',
              description: 'Please wait',
              closeButton: false,
            })
          }}>Loading</Button>
        <Button
          onClick={async () => {
            setTitle('Iman Noah | MSMS')
            await invoke('test_dock_progress')
          }}>Set Title</Button>
        <Button
          onClick={() => {
            toggleDarkMode()
          }}>Toggle dark mode</Button>
      </div>
    </div>
  )
}
