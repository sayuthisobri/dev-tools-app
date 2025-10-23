'use client'
import React, {useEffect, useState} from 'react'
import {Button} from '@/components/ui/button'
import {Input} from '@/components/ui/input'
import {Label} from '@/components/ui/label'
import {Slider} from '@/components/ui/slider'
import {toast} from 'sonner'
import {useAppStore} from '@/stores'
import {PageTitle} from '@/components/ui/typography'
import {run} from '@/lib/utils'
import {listen} from '@/providers/events'

export default function SettingPage(): React.ReactNode {
  const {toggleDarkMode, setTitle, dock, setDockState} = useAppStore()
  const [progressValue, setProgressValue] = useState<number[]>([0])
  const [badgeText, setBadgeText] = useState<string>('')

  useEffect(() => {
    // Listen for dock progress updates
    const unsubscribeProgress = listen('dock-progress-updated', (event) => {
      const dockState = event.payload as { progress: number | null; badge: string | null }
      setDockState(dockState)
      if (dockState.progress !== null) {
        setProgressValue([dockState.progress * 100])
      } else {
        setProgressValue([0])
      }
    })

    // Listen for dock badge updates
    const unsubscribeBadge = listen('dock-badge-updated', (event) => {
      const dockState = event.payload as { progress: number | null; badge: string | null }
      setDockState(dockState)
      if (dockState.badge !== null) {
        setBadgeText(dockState.badge)
      } else {
        setBadgeText('')
      }
    })

    return () => {
      unsubscribeProgress.then(fn => typeof fn == 'function' && fn())
      unsubscribeBadge.then(fn => typeof fn == 'function' && fn())
    }
  }, [setDockState])

  const handleProgressChange = async (value: number[]) => {
    const progress = value[0] / 100
    setProgressValue(value)
    try {
      await run('set_dock_progress', {progress})
    } catch (error) {
      toast.error('Failed to set dock progress')
    }
  }

  const handleSetBadge = async () => {
    try {
      await run('set_dock_badge', {label: badgeText})
    } catch (error) {
      toast.error('Failed to set dock badge')
    }
  }

  const handleClearProgress = async () => {
    try {
      await run('clear_dock')
      setProgressValue([0])
    } catch (error) {
      toast.error('Failed to clear dock progress')
    }
  }

  const handleClearBadge = async () => {
    try {
      await run('clear_dock_badge')
      setBadgeText('')
    } catch (error) {
      toast.error('Failed to clear dock badge')
    }
  }
  return (
    <div className={'m-2 flex flex-col gap-6'}>
      <PageTitle>Settings</PageTitle>

      {/* Dock Settings Section */}
      <div className="space-y-4">
        <h2 className="text-lg font-semibold">Dock Settings</h2>

        {/* Current State Display */}
        <div className="grid grid-cols-2 gap-4 p-4 bg-muted/50 rounded-lg">
          <div>
            <Label className="text-sm font-medium">Current Progress</Label>
            <p
              className="text-2xl font-mono">{dock.progress !== null ? `${Math.round(dock.progress * 100)}%` : 'None'}</p>
          </div>
          <div>
            <Label className="text-sm font-medium">Current Badge</Label>
            <p className="text-2xl font-mono">{dock.badge || 'None'}</p>
          </div>
        </div>

        {/* Progress Control */}
        <div className="space-y-2">
          <Label htmlFor="progress-slider">Dock Progress: {progressValue[0]}%</Label>
          <Slider
            id="progress-slider"
            min={0}
            max={100}
            step={1}
            value={progressValue}
            onValueChange={setProgressValue}
            onValueCommit={handleProgressChange}
            className="w-full"
          />
        </div>

        {/* Badge Control */}
        <div className="space-y-2">
          <Label htmlFor="badge-input">Badge Text</Label>
          <div className="flex gap-2">
            <Input
              id="badge-input"
              value={badgeText}
              onChange={(e) => setBadgeText(e.target.value)}
              placeholder="Enter badge text"
              className="flex-1"
            />
            <Button onClick={handleSetBadge} variant="outline">
              Set Badge
            </Button>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="flex gap-2 flex-wrap">
          <Button onClick={async () => await run('test_dock_progress')} variant="outline">
            Test Animation
          </Button>
          <Button onClick={handleClearProgress} variant="outline">
            Clear Progress
          </Button>
          <Button onClick={handleClearBadge} variant="outline">
            Clear Badge
          </Button>
        </div>
      </div>

      {/* Toast Notification Tests */}
      <div className="space-y-4">
        <h2 className="text-lg font-semibold">Toast Notifications</h2>
        <div className="flex gap-2 flex-wrap">
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
            }}>Set Title</Button>
          <Button
            onClick={() => {
              toggleDarkMode()
            }}>Toggle dark mode</Button>
          <Button
            onClick={async () => {
              setTitle('Iman Noah | MSMS')
            }}>Set Title</Button>
          <Button
            onClick={() => {
              toggleDarkMode()
            }}>Toggle dark mode</Button>
        </div>
      </div>
    </div>
  )
}
