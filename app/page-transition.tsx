'use client'

import {AnimatePresence, motion, Variants} from 'framer-motion'
import {useSelectedLayoutSegment} from 'next/navigation'
import React, {useContext, useEffect, useRef} from 'react'
import {LayoutRouterContext} from 'next/dist/shared/lib/app-router-context.shared-runtime'

function usePreviousValue<T>(value: T): T | undefined {
  const prevValue = useRef<T>()

  useEffect(() => {
    prevValue.current = value
    return () => {
      prevValue.current = undefined
    }
  })

  return prevValue.current
}

function FrozenRouter(props: { children: React.ReactNode }) {
  const context = useContext(LayoutRouterContext)
  const prevContext = usePreviousValue(context) || null

  const segment = useSelectedLayoutSegment()
  const prevSegment = usePreviousValue(segment)

  const changed =
    segment !== prevSegment &&
    segment !== undefined &&
    prevSegment !== undefined

  return (
    <LayoutRouterContext.Provider value={changed ? prevContext : context}>
      {props.children}
    </LayoutRouterContext.Provider>
  )
}

const variants: Variants = {
  hidden: {
    opacity: 0.4,
    // scale: .95,
    x: 50,
    perspective: 80,
  },
  enter: {
    opacity: 1,
    x: 0,
    scale: 1,
    perspective: 90,
    transition: {duration: .3, type: 'spring', bounce: .5},
  },
  exit: {
    opacity: 0.1,
    x: 50,
    // scale: .8,
    transition: {duration: .2, ease: 'easeIn'},
  },
}

export function LayoutTransition(props: {
  children: React.ReactNode;
  className?: React.ComponentProps<typeof motion.div>['className'];
  style?: React.ComponentProps<typeof motion.div>['style'];
  initial: React.ComponentProps<typeof motion.div>['initial'];
  animate: React.ComponentProps<typeof motion.div>['animate'];
  variants: React.ComponentProps<typeof motion.div>['variants'];
  exit: React.ComponentProps<typeof motion.div>['exit'];
}) {
  const segment = useSelectedLayoutSegment()

  return (
    <AnimatePresence mode={'wait'} initial={false}>
      <motion.div
        className={props.className}
        style={props.style}
        key={segment}
        initial={props.initial}
        animate={props.animate}
        variants={props.variants}
        exit={props.exit}
      >
        <FrozenRouter>{props.children}</FrozenRouter>
      </motion.div>
    </AnimatePresence>
  )
}

const PageTransition = ({children}: { children: React.ReactNode }) => {
  // The `key` is tied to the url using the `usePathname` hook.
  const segment = useSelectedLayoutSegment()
  // const key = usePathname()

  return (
    <AnimatePresence mode="popLayout" initial={false}>
      <motion.div
        key={segment}
        initial="hidden"
        animate="enter"
        exit="exit"
        variants={variants}
        transition={{type: 'tween', ease: 'easeOut', duration: .3}}
        className="overflow-hidden h-full"
      >
        <FrozenRouter>{children}</FrozenRouter>
      </motion.div>
    </AnimatePresence>
  )
}

export function MyTransition(props: {
  children: React.ReactNode;
}) {
  return (
    <LayoutTransition
      className="h-full"
      variants={variants}
      initial={'hidden'}
      animate={'enter'}
      exit={'exit'}
    >
      {props.children}
    </LayoutTransition>
  )
}

export default PageTransition
