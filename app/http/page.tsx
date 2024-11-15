'use client'
import React, {useState} from 'react'
import {Popover, PopoverContent, PopoverTrigger} from '@/components/ui/popover'
import Button from '@/components/ui/button'
import {request} from '@/app/http/api'
import {toast} from 'sonner'
import {useForm} from 'react-hook-form'
import {Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage} from '@/components/ui/form'
import {Input} from '@/components/ui/input'
import Debug from 'debug'
import {z} from 'zod'
import {zodResolver} from '@hookform/resolvers/zod'
import {PageTitle} from '@/components/ui/typography'
import {Combobox} from '@/components/ui/combobox'

function PopoverDemo() {
  const [open, setOpen] = useState(false)
  return (
    <Popover modal={false} open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button>{open ? 'Close' : 'Open'}</Button>
      </PopoverTrigger>
      <PopoverContent showArrow={false} side={'right'} title={'Popover title'}>Place content for the popover
        here.</PopoverContent>
    </Popover>

  )
}

const log = Debug('http')
log.color = 'orange'

function HttpForm() {

  const schema = z.object({
    username: z.string().min(2, {
      message: 'Username must be at least 2 characters.',
    }),
  })
  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: {
      username: 'msms',
    },
  })

  function onSubmit(values: z.infer<typeof schema>) {
    log('onSubmit', values)
    form.reset(values)
  }

  return <>
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
        <FormField
          control={form.control}
          name="username"
          render={({field}) => (
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input placeholder="shadcn" {...field} />
              </FormControl>
              <FormDescription>
                This is your public display name.
              </FormDescription>
              <FormMessage/>
            </FormItem>
          )}
        />
        <Button type="submit" disabled={!form.formState.isDirty}>Submit</Button>
      </form>
    </Form>
  </>
}

export default function HttpPage(): React.ReactNode {
  async function sendRequest() {
    const promise = request({
      url: 'https://httpbin.org/post',
      method: 'POST',
      body: JSON.stringify({foo: 'bar'}),
    }).then(res => {
      // const body = res.body && JSON.parse(res.body)
      console.log('res', res)
      return res
    })
    toast.promise(promise, {
      loading: 'Loading',
      success: (d) => `${d.status} ${d.url} body len:${d.length}`,
      error: e => e?.message || 'Failed!',
    })
    return await promise
  }

  const methods = [
    {value: 'GET', label: 'GET'},
    {value: 'POST', label: 'POST'},
    {value: 'PUT', label: 'PUT'},
    {value: 'DELETE', label: 'DELETE'},
    {value: 'PATCH', label: 'PATCH'},
    {value: 'HEAD', label: 'HEAD'},
    {value: 'OPTIONS', label: 'OPTIONS'},
  ]

  return (
    <div className={'p-2 flex gap-2 flex-col'}>
      <PageTitle>HTTP Client</PageTitle>
      <div className="flex gap-2">
        <PopoverDemo/>
        <div>
          <Button onClick={sendRequest}>Test request</Button>
        </div>
        <Combobox data={methods} placeholder='Method'/>
      </div>
      <div className="flex">
        <HttpForm/>
      </div>
    </div>
  )
}
