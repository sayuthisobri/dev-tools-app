'use client'
import React, { useRef } from 'react'
import { PageTitle } from '@/components/ui/typography'
import Button from '@/components/ui/button'
import { startDrag } from '@crabnebula/tauri-plugin-drag'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { z } from 'zod'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import debug from 'debug'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { FileIcon, FolderIcon } from 'lucide-react'
import html2canvas from 'html2canvas'
import { open } from '@tauri-apps/plugin-dialog'

function SetupRemoteForm({className, ...props}: React.ComponentProps<'form'>) {
  const log = debug('setup-remote')
  const schema = z.object({
    host: z.string().min(5, {
      message: 'Please enter a valid host.',
    }),
    port: z.coerce.number().max(65535, 'Please enter a valid port.').step(2),
    username: z.string(),
    password: z.string(),
    key: z.string(),
  })
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      host: '',
      port: 22,
      username: '',
      password: '',
      key: '',
    },
  })

  function onSubmit(values: z.infer<typeof schema>) {
    log('onSubmit', values)
    form.reset(values)
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className={cn('grid items-start gap-4', className)} {...props}>
        <FormField
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
        />
        <Button type="submit">Connect</Button>
        <button className="btn btn-sm btn-info" type="submit">Connect</button>
      </form>
    </Form>
  )
}

export default function FilesPage(): React.ReactNode {

  async function onDragStart(e: React.DragEvent<HTMLOrSVGElement>) {
    e.preventDefault()
    if (!elementRef.current) return
    console.log('drag start', elementRef.current)
    const icon = await html2canvas(elementRef.current, {backgroundColor: null})
      .then((canvas) => canvas.toDataURL('image/png'))
    startDrag({
      icon: icon,
      // item: ['/Users/msms/cdxcore_public_notification_templates.csv']
      item: {
        // data: '<div>hello there!!</div>',
        data: {
          'public.text': 'hello there!!',
          'public.item': '/Users/msms/cdxcore_public_notification_templates.csv',
        },
        types: [
          'public.text',
          'public.item',
          'public.data',
          'public.content',
          // 'public.html',
          'public.xml',
        ],
      },
    }, e => {
      console.log('drag event', e)
    }).then()
  }

  const elementRef = useRef(null)

  return (
    <div className="flex m-2 flex-col gap-2">
      <PageTitle>Remote Files</PageTitle>
      <div className="flex gap-2">
        <Dialog defaultOpen={true}>
          <DialogTrigger asChild>
            <Button variant="outline">New</Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[425px]">
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
          <FolderIcon/>
        </div>

        <div draggable={true} onDrop={e => {
          console.log('drop', e, e.target)
        }} onDragStart={(e) => {
          e.dataTransfer.setData('text/plain', 'test data.com')
          e.dataTransfer.effectAllowed = 'copy'
        }}>
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
    </div>
  )
}
