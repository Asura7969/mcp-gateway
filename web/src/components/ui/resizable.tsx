import * as React from 'react'
import {
  PanelGroup as ResizablePanelGroupPrimitive,
  Panel as ResizablePanelPrimitive,
  PanelResizeHandle as ResizableHandlePrimitive,
} from 'react-resizable-panels'
import { cn } from '@/lib/utils'

type GroupProps = React.ComponentProps<typeof ResizablePanelGroupPrimitive>
type PanelProps = React.ComponentProps<typeof ResizablePanelPrimitive>
type HandleProps = React.ComponentProps<typeof ResizableHandlePrimitive>

function ResizablePanelGroup({ className, ...props }: GroupProps) {
  return (
    <ResizablePanelGroupPrimitive
      data-slot='resizable-panel-group'
      className={cn('flex', className)}
      {...props}
    />
  )
}

function ResizablePanel({ className, ...props }: PanelProps) {
  return (
    <ResizablePanelPrimitive
      data-slot='resizable-panel'
      className={cn('min-h-[300px]', className)}
      {...props}
    />
  )
}

function ResizableHandle({ className, ...props }: HandleProps) {
  return (
    <ResizableHandlePrimitive
      data-slot='resizable-handle'
      className={cn(
        'group relative flex shrink-0 items-center justify-center bg-border/70 transition-colors',
        'data-[panel-group-direction=horizontal]:w-1 data-[panel-group-direction=vertical]:h-1',
        'data-[panel-group-direction=horizontal]:cursor-col-resize data-[panel-group-direction=vertical]:cursor-row-resize',
        'hover:bg-primary/30',
        className
      )}
      {...props}
    >
      {/* Grip indicator: three dots (orientation-aware) */}
      <div
        className={cn(
          'pointer-events-none flex items-center justify-center',
          'data-[panel-group-direction=horizontal]:flex-col data-[panel-group-direction=vertical]:flex-row',
          'gap-0.5'
        )}
      >
        <span className='block rounded-full bg-muted-foreground/50 group-hover:bg-muted-foreground h-[3px] w-[3px]' />
        <span className='block rounded-full bg-muted-foreground/50 group-hover:bg-muted-foreground h-[3px] w-[3px]' />
        <span className='block rounded-full bg-muted-foreground/50 group-hover:bg-muted-foreground h-[3px] w-[3px]' />
      </div>
    </ResizableHandlePrimitive>
  )
}

export { ResizablePanelGroup, ResizablePanel, ResizableHandle }