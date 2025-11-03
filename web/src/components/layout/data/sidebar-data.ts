import {
  LayoutDashboard,
  Monitor,
  Bug,
  HelpCircle,
  Lock,
  Palette,
  ServerOff,
  Settings,
  Wrench,
  UserCog,
  UserX,
  Plug,
  Search,
  Table,
} from 'lucide-react'
import { type SidebarData } from '../types'

export const sidebarData: SidebarData = {
  user: {
    name: 'admin',
    email: 'admin@mcp-gateway.com',
    avatar: '/avatars/shadcn.jpg',
  },
  teams: [], // 移除teams功能，提供空数组以满足类型要求
  navGroups: [
    {
      title: 'General',
      items: [
        {
          title: 'Dashboard',
          url: '/',
          icon: LayoutDashboard,
        },
        {
          title: 'Endpoints',
          url: '/endpoints',
          icon: Plug,
        },
        {
          title: 'Search',
          url: '/search',
          icon: Search,
        },
        {
          title: 'Datasets',
          url: '/datasets',
          icon: Table,
        },
        {
          title: 'Monitoring',
          url: '/monitoring',
          icon: Monitor,
        },
      ],
    },
    {
      title: 'Pages',
      items: [
        {
          title: 'Errors',
          icon: Bug,
          items: [
            {
              title: 'Unauthorized',
              url: '/errors/unauthorized',
              icon: Lock,
            },
            {
              title: 'Forbidden',
              url: '/errors/forbidden',
              icon: UserX,
            },
            {
              title: 'Not Found',
              url: '/errors/not-found',
              icon: ServerOff,
            },
            {
              title: 'Internal Server Error',
              url: '/errors/internal-server-error',
              icon: ServerOff,
            },
            {
              title: 'Maintenance Error',
              url: '/errors/maintenance-error',
              icon: Bug,
            },
          ],
        },
      ],
    },
    {
      title: 'Other',
      items: [
        {
          title: 'Settings',
          icon: Settings,
          items: [
            {
              title: 'Profile',
              url: '/settings',
              icon: UserCog,
            },
            {
              title: 'Account',
              url: '/settings/account',
              icon: Wrench,
            },
            {
              title: 'Appearance',
              url: '/settings/appearance',
              icon: Palette,
            },
          ],
        },
        {
          title: 'Help Center',
          url: '/help-center',
          icon: HelpCircle,
        },
      ],
    },
  ],
}