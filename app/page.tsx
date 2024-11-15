'use client'
// import { invoke } from "@tauri-apps/api/core";
import React, {useEffect, useState} from 'react'
import Monaco from '@/components/ui/monaco'
import {cpuInfo, disks} from 'tauri-plugin-system-info-api'
import {isWebMode} from '@/lib/utils'

export default function Home() {
  const [value, setValue] = useState<string>('// loading..')
  const [cpu, setCpu] = useState<string>('// loading cpu..')
  const [disk, setDisk] = useState<string>('// loading disk..')
  useEffect(() => {
    if (isWebMode()) return
    cpuInfo().then((res) => {
      setCpu(`const cpuDetails = ` + JSON.stringify(res, null, 2))
    }).catch(e => {
      console.error('opps', e)
    })

    disks().then((res) => {
      setDisk(`const diskDetails = ` + JSON.stringify(res, null, 2))
    })
  }, [])

  useEffect(() => {
    setValue(`
//-----CPU
${cpu}
    
//-----DISK
${disk}
`)
  }, [cpu, disk])

  return (
    <div className="flex flex-col gap-2 h-full">
      {/*<PageTitle className={'mt-2 mx-2'}>Home</PageTitle>*/}
      <Monaco value={value} language={'javascript'}/>
    </div>
  )
}

