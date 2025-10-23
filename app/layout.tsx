'use client'
import localFont from 'next/font/local'
import './globals.css'
import React, {useEffect} from 'react'
import {ThemeProvider as NextThemesProvider} from 'next-themes'
import {useAppStore} from '@/stores'
import Sidebar from '@/components/ui/sidebar'
import {TooltipProvider} from '@/components/ui/tooltip'
import {Toaster} from '@/components/ui/sonner'
import Titlebar from '@/components/ui/title-bar'
import {CommandMenu} from '@/app/command'
import {listen} from '@tauri-apps/api/event'
import {isWebMode, stringifyWithRefs} from '@/lib/utils'
import {useRouter} from 'next/navigation'
import {currentMonitor, getCurrentWindow, PhysicalPosition} from '@tauri-apps/api/window'
import {useFileDrop} from '@/hooks/file-drop'
import {MyTransition} from '@/app/page-transition'

import {debug, error, info, trace, warn} from '@tauri-apps/plugin-log';
import '@svar-ui/react-grid/all.css';
import {isObjectLike} from 'lodash'

const geistSans = localFont({
  src: '../assets/fonts/GeistVF.woff',
  variable: '--font-geist-sans',
  weight: '100 900',
})
const geistMono = localFont({
  src: '../assets/fonts/GeistMonoVF.woff',
  variable: '--font-geist-mono',
  weight: '100 900',
})

export default function RootLayout({
                                     children,
                                   }: Readonly<{ children: any }>) {
  const {isDarkMode, isSidebarCollapsed, isCollapsible} = useAppStore()
  const router = useRouter()

  async function init() {

    function forwardConsole(
      fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
      logger: (message: string) => Promise<void>,
    ) {
      const original = console[fnName];
      console[fnName] = (...message) => {
        original(...message);
        logger(message.map(m => isObjectLike(m) ? stringifyWithRefs(m) : m).join(', ')).then();
      };
    }

    if (!(console as any)['is-init']) {
      forwardConsole('log', trace);
      forwardConsole('debug', debug);
      forwardConsole('info', info);
      forwardConsole('warn', warn);
      forwardConsole('error', error);
      (console as any)['is-init'] = true;
    }
    const monitor = await currentMonitor()
    if (monitor != null) {
      const currentWindow = getCurrentWindow()
      let size = Object.assign({}, monitor.size)
      const scale = 0.8
      size.width = Math.round(size.width * scale)
      size.height = Math.round(size.height * scale)
      const physicalPosition = new PhysicalPosition(
        Math.round((monitor.size.width - size.width) / 2),
        Math.round((monitor.size.height - size.height) / 2),
      )
      console.log('position', physicalPosition, monitor.size, size)
      // await currentWindow.setPosition(physicalPosition)
      // await currentWindow.setSize(size)
    }

  }

  useEffect(() => {
    if (isWebMode()) return
    init().then()
    let unListen: () => void
    // set up event listener
    listen('go-to', event => {
      if (typeof event.payload != 'string') return
      const payload: string = event.payload as string
      if (payload.includes('::')) {
        const [section, query] = payload.split('::', 2)
        switch (query) {
          case 'refresh':
            window.location.reload()
            break
          default:
            console.log('unhandled goto', payload)
        }
      } else {
        router.push(payload)
      }
    }).then(v => unListen = v)


    return () => {
      // cleanup
      if (!!unListen) unListen()
    }
  })
  useFileDrop()


  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={isDarkMode ? 'dark' : 'light'}
    >
    <body
      onContextMenu={(e) => e.preventDefault()}
      className={`${geistSans.variable} ${geistMono.variable} antialiased select-none overflow-hidden
      
      `}
    >
    <NextThemesProvider
      attribute="class"
      defaultTheme="dark"
      disableTransitionOnChange
    >
      <TooltipProvider>
        <Titlebar/>
        <div
          className={`flex text-gray-900 w-full pt-7 pl-16`}
        >
          <Sidebar/>
          <main
            className={`grow h-[calc(100vh-1.75rem)] bg-background/50 overflow-auto
            `}
          >
            <MyTransition>{children}</MyTransition>
            {/*{children}*/}
          </main>
        </div>
      </TooltipProvider>
      <Toaster toastOptions={{
        duration: 1000 * 60 * 3,
        closeButton: true,
      }}/>
      <CommandMenu/>
    </NextThemesProvider>
    </body>
    </html>
  )
}
