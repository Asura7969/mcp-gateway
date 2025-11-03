import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

function Breadcrumb({ className, ...props }: React.ComponentProps<'nav'>) {
  return (
    <nav
      aria-label='breadcrumb'
      className={cn('text-sm', className)}
      {...props}
    />
  )
}

function BreadcrumbList({ className, ...props }: React.ComponentProps<'ol'>) {
  return (
    <ol
      className={cn('flex items-center gap-1 text-muted-foreground', className)}
      {...props}
    />
  )
}

function BreadcrumbItem({ className, ...props }: React.ComponentProps<'li'>) {
  return <li className={cn('inline-flex items-center gap-1', className)} {...props} />
}

function BreadcrumbSeparator({ className, ...props }: React.ComponentProps<'span'>) {
  return (
    <span role='presentation' className={cn('inline-flex', className)} {...props}>
      <ChevronRight className='size-3.5' />
    </span>
  )
}

function BreadcrumbLink({ asChild = false, className, ...props }: React.ComponentProps<'a'> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : 'a'
  return (
    <Comp
      className={cn('text-foreground hover:underline underline-offset-4', className)}
      {...props}
    />
  )
}

function BreadcrumbPage({ className, ...props }: React.ComponentProps<'span'>) {
  return (
    <span
      aria-current='page'
      className={cn('text-foreground font-medium', className)}
      {...props}
    />
  )
}

export {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbSeparator,
  BreadcrumbLink,
  BreadcrumbPage,
}