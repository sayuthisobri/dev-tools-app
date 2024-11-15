'use client'
import React, {useEffect} from 'react'
import {Popover, PopoverContent, PopoverTrigger} from '@/components/ui/popover'
import Button from '@/components/ui/button'
import {Check, ChevronsUpDown} from 'lucide-react'
import {Command, CommandEmpty, CommandGroup, CommandInput, CommandItem, CommandList} from '@/components/ui/command'
import {cn} from '@/lib/utils'
import {ClassValue} from 'clsx'

const frameworks = [
  {
    value: 'next.js',
    label: 'Next.js',
  },
  {
    value: 'sveltekit',
    label: 'SvelteKit',
  },
  {
    value: 'nuxt.js',
    label: 'Nuxt.js',
  },
  {
    value: 'remix',
    label: 'Remix',
  },
  {
    value: 'astro',
    label: 'Astro',
  },
]

interface ValueData<T> {
  value: T
  label: string
}

export interface ComboProps<T> {
  defaultValue?: T
  data: ValueData<T>[]
  value?: T
  onChange?: (id: string, value: T) => void
  placeholder?: string
  searchPlaceholder?: string
  emptyNode?: React.ReactNode
  width?: string
  triggerClass?: ClassValue
  idFn?: (item: T) => string
}

export function Combobox<T>(
  {
    placeholder = 'Select..',
    idFn,
    data,
    defaultValue,
    value,
    onChange,
    ...props
  }: ComboProps<T>) {
  const [open, setOpen] = React.useState(false)
  if (!idFn) {
    idFn = (item) => typeof item === 'string' ? item :
      (!!item && typeof item === 'object' && item.hasOwnProperty('id')) ? (item as any)['id'] :
        JSON.stringify(item)
  }
  const [id, setId] = React.useState<string>(defaultValue ? idFn(defaultValue) : '')
  useEffect(() => {
    if(!value) {
      setId('')
      return
    }
    const _id = idFn(value)
    if(id!==_id) {
      setId(_id)
    }
  }, [value])
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn('justify-between', props.triggerClass)}
        >
          {id
            ? data.find((o) => idFn(o.value) === id)?.label
            : placeholder || 'Select..'}
          <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50"/>
        </Button>
      </PopoverTrigger>
      <PopoverContent className={`w-[${props.width || '200px'}] p-0`} sideOffset={0} showArrow={false}>
        <Command>
          <CommandInput placeholder={props.searchPlaceholder || 'Search'}/>
          <CommandList>
            <CommandEmpty>{props.emptyNode || 'No items found.'}</CommandEmpty>
            <CommandGroup>
              {data.map((item) => {
                const itemId = idFn(item.value)
                return <CommandItem
                  key={itemId}
                  value={itemId}
                  onSelect={(current) => {
                    console.log(current, itemId, item, id)
                    setId(current === id ? '' : current)
                    onChange && onChange(itemId, item.value)
                    setOpen(false)
                  }}
                >
                  <Check
                    className={cn(
                      'mr-2 h-4 w-4',
                      id === itemId ? 'opacity-100' : 'opacity-0',
                    )}
                  />
                  {item.label}
                </CommandItem>
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}