'use client'
import React, {useEffect, useRef, useState} from 'react'
import {PageTitle} from '@/components/ui/typography'
import Button from '@/components/ui/button'
import {startDrag} from '@crabnebula/tauri-plugin-drag'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {Input} from '@/components/ui/input'
import {cn} from '@/lib/utils'
import {z} from 'zod'
import {useForm} from 'react-hook-form'
import {zodResolver} from '@hookform/resolvers/zod'
import debug from 'debug'
import {Form, FormControl, FormField, FormItem, FormLabel, FormMessage} from '@/components/ui/form'
import {FileArchiveIcon, FileIcon} from 'lucide-react'
import {open} from '@tauri-apps/plugin-dialog'
import {Combobox} from '@/components/ui/combobox'
import {invoke} from '@tauri-apps/api/core'
import html2canvas from 'html2canvas-pro'
import {Grid, IColumnConfig} from '@svar-ui/react-grid'
import {WillowDark} from '@svar-ui/react-core'

function SetupRemoteForm({className, ...props}: React.ComponentProps<'form'>) {
  const log = debug('setup-remote')
  const schema = z.object({
    type: z.enum(['sftp', 's3']),
    host: z.string().min(5, {
      message: 'Please enter a valid host.',
    }),
    port: z.coerce.number().max(65535, 'Please enter a valid port.').step(2),
    profile: z.string(),
    username: z.string(),
    password: z.string(),
    key: z.string(),
  })
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      type: 's3',
      host: '',
      port: 22,
      username: '',
      password: '',
      key: '',
    },
  })
  const [profiles, setProfiles] = useState<string[]>([])
  useEffect(() => {
    async function loadProfiles() {
      const profiles = await invoke('aws_profiles', {path: '~/.aws/config'})
      setProfiles(Array.isArray(profiles) ? profiles : [])
      log('profiles', profiles)
    }

    loadProfiles().then()
  }, [])

  function onSubmit(values: z.infer<typeof schema>) {
    log('onSubmit', values)
    form.reset(values)
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className={cn('grid items-start gap-4', className)} {...props}>
        <FormField
          control={form.control}
          name="type"
          render={({field}) => (
            <FormItem>
              <FormLabel>Type</FormLabel>
              <FormControl>
                <Combobox data={[{
                  label: 'sftp',
                  value: 'sftp',
                }, {
                  label: 's3',
                  value: 's3',
                }]} value={field.value} onChange={field.onChange} triggerClass={'w-full'}/>
              </FormControl>
              <FormMessage/>
            </FormItem>
          )}
        />
        {form.watch('type') === 's3' && <>
          <FormField
            control={form.control}
            name="profile"
            render={({field}) => (
              <FormItem>
                <FormLabel>Profile</FormLabel>
                <FormControl>
                  <Combobox data={profiles.map(p => ({label: p, value: p})) || []}
                            value={field.value}
                            onChange={field.onChange}
                            triggerClass={'w-full'}/>
                </FormControl>
                <FormMessage/>
              </FormItem>
            )}
          />
        </>}
        {form.watch('type') === 'sftp' && <><FormField
          control={form.control}
          name="host"
          render={({field}) => (
            <FormItem>
              <FormLabel>Host</FormLabel>
              <FormControl>
                <Input placeholder="msms.work:22" {...field} />
              </FormControl>
              <FormMessage/>
            </FormItem>
          )}
        />
          <FormField
            control={form.control}
            name="port"
            render={({field}) => (
              <FormItem>
                <FormLabel>Port</FormLabel>
                <FormControl>
                  <Input type="number" placeholder="22" {...field} />
                </FormControl>
                <FormMessage/>
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="username"
            render={({field}) => (
              <FormItem>
                <FormLabel>Username</FormLabel>
                <FormControl>
                  <Input {...field} />
                </FormControl>
                <FormMessage/>
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="password"
            render={({field}) => (
              <FormItem>
                <FormLabel>Password</FormLabel>
                <FormControl>
                  <Input type="password" {...field} />
                </FormControl>
                <FormMessage/>
              </FormItem>
            )}
          /></>}
        <Button className="btn btn-sm" variant={'primary'} type="submit">Connect</Button>
      </form>
    </Form>
  )
}

export default function FilesPage(): React.ReactNode {

  async function onDragStart(e: React.DragEvent<HTMLOrSVGElement>) {
    e.preventDefault()
    if (!elementRef.current) return
    // console.log('drag start', elementRef.current)
    const icon = await html2canvas(e.target as HTMLElement, {backgroundColor: null})
      .then((canvas) => canvas.toDataURL('image/png'))
    startDrag({
      // icon: '/Users/msms/workspaces/msms/desktop-app/tauri-dev-tools/src-tauri/icons/128x128.png',
      icon: icon,
      // item: ['/Users/msms/cdxcore_public_notification_templates.csv']
      item: ['/Users/msms/Downloads/ims-log.txt'],
    }, e => {
      console.log('drag event', e)
    }).then()
  }

  const elementRef = useRef(null)


  const data = [
    {
      id: 1,
      city: 'Amieshire',
      email: 'Leora13@yahoo.com',
      firstName: 'Ernest',
      lastName: 'Schuppe',
      companyName: 'Lebsack - Nicolas',
    },
    {
      id: 2,
      city: 'Gust',
      email: 'Mose_Gerhold51@yahoo.com',
      firstName: 'Janis',
      lastName: 'Vandervort',
      companyName: 'Glover - Hermiston',
    },
  ];

  const columns: IColumnConfig[] = [
    {id: 'id', width: 50, draggable: true},
    {id: 'city', width: 100, header: 'City'},
    {id: 'firstName', header: 'First Name', width: 150, sort: true, resize: true},
    {id: 'lastName', header: 'Last Name', width: 150, sort: true, resize: true},
    {id: 'email', header: 'Email'},
    {id: 'companyName', header: 'Company'},
  ];
  return (
    <div className="flex m-2 flex-col gap-2">
      <PageTitle>Remote Files</PageTitle>
      <div className="flex gap-2">
        <Dialog defaultOpen={true}>
          <DialogTrigger asChild>
            <Button variant="outline">New</Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[425px]" aria-describedby="setup-remote-form">
            <DialogHeader>
              <DialogTitle>New remote</DialogTitle>
              <DialogDescription>
                Setup new remote configuration you wish to connect.
              </DialogDescription>
            </DialogHeader>
            <SetupRemoteForm/>
          </DialogContent>
        </Dialog>

        <div ref={elementRef} draggable={true} onDragStart={onDragStart}>
          <FileArchiveIcon/>
        </div>

        <div draggable={true} onDrop={e => {
          console.log('drop', e, e.target)
        }} onDragStart={onDragStart}>
          <Button onClick={async e => {
            e.preventDefault()
            const res = await open({
              multiple: true,
              directory: true,
            })

            console.log('selected', res)
          }}>
            <FileIcon className="text-red-200"/>
          </Button>
        </div>
      </div>
      <div>

        <div className={""}>
          <WillowDark>
            <Grid data={data} columns={columns} reorder={false}/>
          </WillowDark>
        </div>
      </div>
    </div>
  )
}
