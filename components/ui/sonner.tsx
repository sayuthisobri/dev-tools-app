'use client'

import {useTheme} from 'next-themes'
import {Toaster as Sonner} from 'sonner'
import {cn} from '@/lib/utils'

type ToasterProps = React.ComponentProps<typeof Sonner>

const Toaster = ({toastOptions, className, ...props}: ToasterProps) => {
  const {theme = 'system'} = useTheme()

  return (
    <Sonner
      theme={theme as ToasterProps['theme']}
      className={cn('toaster', className)}
      toastOptions={Object.assign({
        // unstyled: true,
        classNames: {
          error: 'bg-red-500 text-red-100 border-red-500',
          success: 'bg-green-300 text-green-600 border-green-400',
          warning: 'bg-orange-500 text-orange-800 border-orange-500',
          info: 'bg-blue-500 text-blue-800 border-blue-500/40',
          toast: 'bg-opacity-90 border-1 shadow shadow-gray-400/70',
          loading: 'bg-purple-900 text-purple-200 border-purple-700',
          // title: 'text-gray-600',
          icon: 'icon',
          description: 'text-xs text-opacity-70 description',
          actionButton: 'bg-zinc-400',
          cancelButton: 'bg-orange-400',
          closeButton: 'bg-gray-600/60 border-0 shadow',
          content: 'mr-auto',
        },
      }, toastOptions)}
      {...props}
    />
  )
}

export { Toaster }
