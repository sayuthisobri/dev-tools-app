'use client'
import localFont from 'next/font/local'
import './globals.css'
import React from 'react'
import {ThemeProvider as NextThemesProvider} from 'next-themes'
import {useAppStore} from '@/stores'
import Sidebar from '@/components/ui/sidebar'
import {TooltipProvider} from '@/components/ui/tooltip'
import {Toaster} from '@/components/ui/sonner'
import {MyTransition} from '@/app/page-transition'
import Titlebar from '@/components/ui/title-bar'
import {CommandMenu} from '@/app/command'

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


  return (
    <html
      lang="en"
      suppressHydrationWarning
      className={isDarkMode ? 'dark' : 'light'}
    >
    <body
      onContextMenu={(e) => e.preventDefault()}
      className={`${geistSans.variable} ${geistMono.variable} antialiased select-none`}
    >
    <NextThemesProvider
      attribute="class"
      enableSystem
      disableTransitionOnChange
    >
      <TooltipProvider>
        <Titlebar/>
        <div
          className={`flex text-gray-900 w-full pt-7 pl-16`}
        >
          <Sidebar/>
          <main
            className={`grow h-[calc(100vh-1.75rem)] bg-background/50
            `}
          >
            <MyTransition>{children}</MyTransition>
          </main>
        </div>
      </TooltipProvider>
      <Toaster toastOptions={{
        duration: 1000 * 60 * 3,
        closeButton: true,
      }}/>
      <CommandMenu />
    </NextThemesProvider>
    </body>
    </html>
  )
}
