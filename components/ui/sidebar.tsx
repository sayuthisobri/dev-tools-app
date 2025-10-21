'use client'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useAppStore } from '@/stores'
import { motion } from 'framer-motion'
import {
  CloudyIcon,
  CodeXmlIcon,
  ContainerIcon,
  HardDriveIcon,
  Layout,
  LucideIcon,
  Menu,
  Send,
  SlidersHorizontal
} from 'lucide-react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'

interface SidebarLinkProps {
  href: string;
  icon: LucideIcon;
  label: string;
}

const SidebarLink = ({href, icon: Icon, label}: SidebarLinkProps) => {
  const pathname = usePathname()
  const isActive =
    pathname === href

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Link href={href}>
          <motion.div
            whileTap={{
              scale: 0.8,
              transition: {
                duration: 0.2,
              },
            }}
            className={`cursor-pointer flex items-center justify-center py-4
        hover:text-pink-500 gap-3 transition-colors duration-200 ease-out ${
              isActive ? 'bg-gray-50 bg-opacity-30 text-pink-400' : ''
            }
        `}
          >
            <Icon className="w-6 h-6"/>

            {/*    <span*/}
            {/*      className={`font-medium text-gray-700`}*/}
            {/*    >*/}
            {/*  {label}*/}
            {/*</span>*/}
          </motion.div>
        </Link>
      </TooltipTrigger>
      <TooltipContent side={'right'}>
        {label}
      </TooltipContent>
    </Tooltip>
  )
}

const Sidebar = () => {
  const {
    isSidebarCollapsed,
    isCollapsible,
    setIsSidebarCollapsed,
  } = useAppStore()

  const toggleSidebar = () => {
    setIsSidebarCollapsed(!isSidebarCollapsed)
  }

  return (
    <div className={`fixed flex flex-col w-16 left-0
  bg-gray-400/50 
  transition-all duration-300
  overflow-hidden h-full shadow-md z-40`}>
      <div
        className={`flex gap-3 justify-between md:justify-normal items-center pt-8 px-5`}
      >
        <HardDriveIcon className="w-6 h-6 text-blue-500"/>
        <h1
          className={`${
            isSidebarCollapsed || !isCollapsible ? 'scale-0 hidden' : 'block scale-100'
          } font-extrabold text-2xl transition-transform duration-500 ease-out`}
        >
          DevTools
        </h1>
        {isCollapsible &&
          <button
            className="md:hidden px-3 py-3 bg-gray-100 rounded-full hover:bg-blue-100"
            onClick={toggleSidebar}
          >
            <Menu className="w-4 h-4"/>
          </button>}
      </div>

      {/* LINKS */}
      <div className="flexflex-grow mt-8">
        <SidebarLink
          href="/"
          icon={Layout}
          label="Dashboard"
        />
        <SidebarLink
          href="/http"
          icon={Send}
          label="HTTP Client"
        />
        <SidebarLink
          href="/kube"
          icon={ContainerIcon}
          label="K8s"
        />
        <SidebarLink
          href="/soap"
          icon={CodeXmlIcon}
          label="SOAP"
        />
        <SidebarLink
          href="/files"
          icon={CloudyIcon}
          label="Remote Files"
        />
        <SidebarLink
          href="/settings"
          icon={SlidersHorizontal}
          label="Settings"
        />
      </div>

      {/* FOOTER */}
      <div className={`${isSidebarCollapsed || !isCollapsible ? 'hidden' : 'block'} mb-10`}>
        <p className="text-center text-xs text-gray-500">&copy; 2024 msms</p>
      </div>
    </div>
  )
}

export default Sidebar
