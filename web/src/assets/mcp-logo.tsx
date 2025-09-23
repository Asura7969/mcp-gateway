import { type SVGProps } from 'react'
import { cn } from '@/lib/utils'

export function McpLogo({ className, ...props }: SVGProps<SVGSVGElement>) {
  return (
    <svg
      id='mcp-gateway-logo'
      viewBox='0 0 24 24'
      xmlns='http://www.w3.org/2000/svg'
      height='24'
      width='24'
      className={cn('size-6', className)}
      {...props}
    >
      <title>MCP Gateway</title>
      <defs>
        <linearGradient id="techGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#0ea5e9" />
          <stop offset="50%" stopColor="#3b82f6" />
          <stop offset="100%" stopColor="#6366f1" />
        </linearGradient>
      </defs>
      
      {/* 科技感六边形外框 */}
      <polygon 
        points="12,2 21,7 21,17 12,22 3,17 3,7" 
        fill="none" 
        stroke="url(#techGradient)" 
        strokeWidth="1.5"
      />
      
      {/* 内部简约电路图案 */}
      <path 
        d="M8 12h8M12 8v8" 
        stroke="url(#techGradient)" 
        strokeWidth="2" 
        strokeLinecap="round"
      />
      
      {/* 数据节点 */}
      <circle cx="8" cy="8" r="1.5" fill="#0ea5e9" />
      <circle cx="16" cy="8" r="1.5" fill="#3b82f6" />
      <circle cx="8" cy="16" r="1.5" fill="#3b82f6" />
      <circle cx="16" cy="16" r="1.5" fill="#6366f1" />
      <circle cx="12" cy="12" r="2" fill="url(#techGradient)" />
    </svg>
  )
}