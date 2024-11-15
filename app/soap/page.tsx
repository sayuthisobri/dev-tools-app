'use client'

import {useEffect} from 'react'
import Button from '@/components/ui/button'
import {Command} from 'tauri-plugin-shellx-api'
import {
  Drawer,
  DrawerClose,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
  DrawerTrigger,
} from '@/components/ui/drawer'
import {toast} from 'sonner'
import {PageTitle} from '@/components/ui/typography'

function DrawerDemo() {
  return (
    <Drawer>
      <DrawerTrigger>Open</DrawerTrigger>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>Are you absolutely sure?</DrawerTitle>
          <DrawerDescription>This action cannot be undone.</DrawerDescription>
        </DrawerHeader>
        <DrawerFooter>
          <Button>Submit</Button>
          <DrawerClose>
            <Button variant="outline">Cancel</Button>
          </DrawerClose>
        </DrawerFooter>
      </DrawerContent>
    </Drawer>

  )
}


export default function SoapPage() {
  useEffect(() => {
  })
  return <div className="flex gap-2 flex-col m-2">
    <PageTitle>SOAP Client</PageTitle>
    <div className="flex gap-2"><DrawerDemo/>
      <Button onClick={() => {
        const cmd = Command.create('aws', ['sts', 'get-caller-identity', '--profile', 'CloudEngineer-411632713503'])
        const res = cmd.execute().then(x => {
          if (x.code !== 0) {
            throw x
          }
          return `${x.code}: ${x.stdout || x.stderr}`
        }).catch(x => {
          throw x.stderr || x?.toString()
        })
        toast.promise(res,
          {
            loading: 'Checking AWS',
            success: (d) => d || 'OK',
            error: e => e || 'Failed!',
          },
        )
      }}>Check AWS</Button></div>
  </div>
}