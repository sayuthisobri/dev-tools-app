'use client'
// import { invoke } from "@tauri-apps/api/core";
import React, {useEffect, useState} from 'react'
import {cpuInfo, disks} from 'tauri-plugin-system-info-api'
import MonacoEditor from '@/components/ui/monaco'
import {PageTitle} from '@/components/ui/typography'

export default function Home() {
  const [value, setValue] = useState<string>('// loading..')
  const [cpu, setCpu] = useState<string>('// loading cpu..')
  const [disk, setDisk] = useState<string>('// loading disk..')
  const [env, setEnv] = useState<string>('// loading env..')
  useEffect(() => {
    // if (isWebMode()) return
    cpuInfo().then((res) => {
      setCpu(`const cpuDetails = ` + JSON.stringify(res, null, 2))
    }).catch(e => {
      console.error('opps', e)
    })

    // run('env').then((res) => {
    //   console.info('env', res)
    //   setEnv(`const envDetails = ` + JSON.stringify(res, null, 2))
    // }).catch(x=>{
    //   console.error('opps', x)
    // })

    disks().then((res) => {
      setDisk(`const diskDetails = ` + JSON.stringify(res, null, 2))
    })
  }, [])

  useEffect(() => {
    setValue(`
//-----ENV
${env}

//-----CPU
${cpu}
    
//-----DISK
${disk}
`)
  }, [cpu, disk, env])

  return (
    <div className="flex flex-col gap-2 h-full">
      <PageTitle className={'mt-2 mx-2'}>Home</PageTitle>
      <MonacoEditor value={value} language={'javascript'}/>
    </div>
  )
}

