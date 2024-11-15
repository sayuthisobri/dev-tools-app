'use client'
import * as React from 'react'
import {forwardRef} from 'react'
import {Slot} from '@radix-ui/react-slot'
import {cva, type VariantProps} from 'class-variance-authority'

import {cn} from '@/lib/utils'
import {motion, MotionProps} from 'framer-motion'

const buttonVariants = cva(
  'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0',
  {
    variants: {
      variant: {
        default: 'bg-primary/60 text-primary-foreground hover:bg-primary/90',
        primary: 'bg-blue-400/80 text-gray-900 hover:bg-blue-500/90',
        warning: 'bg-yellow-400/80 text-gray-900 hover:bg-yellow-500/90',
        success: 'bg-green-400/80 text-gray-900 hover:bg-green-500/90',
        error: 'bg-red-400/80 text-gray-900 hover:bg-red-500/90',
        destructive: 'bg-red-400/80 text-gray-900 hover:bg-red-500/90',
        outline:
          'border border-gray-200 bg-background/40 hover:text-accent-foreground',
        secondary:
          'bg-secondary text-secondary-foreground hover:bg-secondary/80',
        ghost: 'hover:bg-accent hover:text-accent-foreground',
        link: 'text-primary underline-offset-4 hover:underline',
      },
      size: {
        default: 'h-10 px-4 py-2',
        sm: 'h-8 rounded-md px-3',
        lg: 'h-11 rounded-md px-8',
        icon: 'h-10 w-10',
      },
    },
    defaultVariants: {
      variant: 'default',
      size: 'sm',
    },
  },
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const NormalButton = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({className, variant, size, asChild = false, ...props}, ref) => {
    const Comp = asChild ? Slot : 'button'
    return (
      <Comp
        className={cn(buttonVariants({variant, size, className}))}
        ref={ref}
        {...props}
      />
    )
  },
)
NormalButton.displayName = 'Button'

const AnimatedButton_ = motion.create(NormalButton)

const Button = forwardRef<HTMLButtonElement, MotionProps & ButtonProps>(({
                                                                           children,
                                                                           whileHover,
                                                                           whileTap,
                                                                           ...props
                                                                         }: MotionProps & ButtonProps, ref) => {
  return (
    <AnimatedButton_
      className={'shad'}
      ref={ref}
      whileHover={whileHover ?? {
        transition: {duration: 0.1, type: 'spring'},
        boxShadow: '0 0 3px 1px hsl(var(--twc-gray-600) / .3)',
      }}
      whileTap={whileTap ?? {
        scale: 0.9,
        boxShadow: '0 0 5px 2px hsl(var(--twc-gray-300) / .3)',
      }} {...props}>{children}</AnimatedButton_>
  )
})

export { Button, NormalButton, buttonVariants }
export default Button