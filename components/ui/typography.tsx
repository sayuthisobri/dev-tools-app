import React, {forwardRef} from 'react'
import {TextAnimate, TextAnimationProps} from '@/components/ui/text-animate'
import {cn} from '@/lib/utils'

export const PageTitle = forwardRef<React.ElementRef<typeof TextAnimate>,
  React.ComponentPropsWithoutRef<typeof TextAnimate>>(
  ({
     children,
     className,
     type,
     ...props
   }: TextAnimationProps, ref) => {
    return (
      <TextAnimate ref={ref} type={type || 'rollIn'} {...props}
                   className={cn('text-xl text-blue-500 font-bold', className)}>
        {children}
      </TextAnimate>
    )
  })
