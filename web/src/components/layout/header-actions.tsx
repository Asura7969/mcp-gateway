import { cn } from '@/lib/utils'
import { ProfileDropdown } from '@/components/profile-dropdown'
import { ThemeSwitch } from '@/components/theme-switch'
import { ConfigDrawer } from '@/components/config-drawer'

type HeaderActionsProps = React.HTMLAttributes<HTMLDivElement>

export function HeaderActions({ className, ...props }: HeaderActionsProps) {
  return (
    <div
      className={cn('ms-auto flex items-center space-x-4', className)}
      {...props}
    >
      <ThemeSwitch />
      <ConfigDrawer />
      <ProfileDropdown />
    </div>
  )
}