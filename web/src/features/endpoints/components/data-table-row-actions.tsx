import { useState, useEffect, useRef } from 'react'
import { type Row } from '@tanstack/react-table'
import { MoreHorizontal, Copy } from 'lucide-react'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ChevronDown, ChevronRight } from 'lucide-react'
import SyntaxHighlighter from 'react-syntax-highlighter'
import { github } from 'react-syntax-highlighter/dist/esm/styles/hljs'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'
import { type Endpoint } from '../data/schema'
import { EndpointsApiService } from '../data/api'

// 创建深色主题样式
const darkSyntaxStyle = {
  hljs: {
    background: '#1f1f23',
    color: '#f0f0f0'
  },
  'hljs-subst': {
    color: '#f0f0f0'
  },
  'hljs-comment': {
    color: '#a0a0a0'
  },
  'hljs-keyword': {
    color: '#4096ff'
  },
  'hljs-attribute': {
    color: '#4096ff'
  },
  'hljs-selector-tag': {
    color: '#4096ff'
  },
  'hljs-meta-keyword': {
    color: '#4096ff'
  },
  'hljs-doctag': {
    color: '#4096ff'
  },
  'hljs-name': {
    color: '#4096ff'
  },
  'hljs-built_in': {
    color: '#52c41a'
  },
  'hljs-literal': {
    color: '#52c41a'
  },
  'hljs-bullet': {
    color: '#52c41a'
  },
  'hljs-code': {
    color: '#52c41a'
  },
  'hljs-addition': {
    color: '#52c41a'
  },
  'hljs-regexp': {
    color: '#faad14'
  },
  'hljs-symbol': {
    color: '#faad14'
  },
  'hljs-variable': {
    color: '#faad14'
  },
  'hljs-template-variable': {
    color: '#faad14'
  },
  'hljs-link': {
    color: '#faad14'
  },
  'hljs-selector-attr': {
    color: '#faad14'
  },
  'hljs-selector-pseudo': {
    color: '#faad14'
  },
  'hljs-type': {
    color: '#722ed1'
  },
  'hljs-string': {
    color: '#722ed1'
  },
  'hljs-number': {
    color: '#722ed1'
  },
  'hljs-selector-id': {
    color: '#722ed1'
  },
  'hljs-selector-class': {
    color: '#722ed1'
  },
  'hljs-quote': {
    color: '#722ed1'
  },
  'hljs-template-tag': {
    color: '#722ed1'
  },
  'hljs-deletion': {
    color: '#ff4d4f'
  },
  'hljs-title': {
    color: '#ff4d4f'
  },
  'hljs-section': {
    color: '#ff4d4f'
  },
  'hljs-function': {
    color: '#ff4d4f'
  },
  'hljs-meta': {
    color: '#ff4d4f'
  },
  'hljs-emphasis': {
    fontStyle: 'italic'
  },
  'hljs-strong': {
    fontWeight: 'bold'
  }
}

// JSON高亮组件
const JsonHighlighter = ({ children, className = "" }: { children: string; className?: string }) => {
  const isDarkMode = document.documentElement.classList.contains('dark')
  
  return (
    <SyntaxHighlighter 
      language="json" 
      style={isDarkMode ? darkSyntaxStyle : github} 
      className={className}
      customStyle={{ 
        backgroundColor: 'inherit',
        margin: 0,
        padding: '12px' // 保持p-3的padding
      }}
    >
      {children}
    </SyntaxHighlighter>
  )
}

interface DataTableRowActionsProps {
  row: Row<Endpoint>
}

export function DataTableRowActions({ row }: DataTableRowActionsProps) {
  const [isViewOpen, setIsViewOpen] = useState(false)
  const [isEditOpen, setIsEditOpen] = useState(false)
  const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false)
  const [endpointDetail, setEndpointDetail] = useState<any>(null)
  const [openApiDetails, setOpenApiDetails] = useState<Record<string, boolean>>({})
  const [isDeleting, setIsDeleting] = useState(false)
  const [copied, setCopied] = useState(false)
  const swaggerTextareaRef = useRef<HTMLTextAreaElement>(null)

  // 添加ESC键关闭功能
  useEffect(() => {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsViewOpen(false)
        setIsEditOpen(false)
      }
    }

    window.addEventListener('keydown', handleEsc)
    return () => {
      window.removeEventListener('keydown', handleEsc)
    }
  }, [])

  const handleView = async () => {
    try {
      const detail = await EndpointsApiService.getEndpointById(row.original.id)
      setEndpointDetail(detail)
      setIsViewOpen(true)
    } catch (error) {
      console.error('Failed to fetch endpoint detail:', error)
    }
  }

  const handleEdit = async () => {
    try {
      const detail = await EndpointsApiService.getEndpointById(row.original.id)
      setEndpointDetail(detail)
      setIsEditOpen(true)
    } catch (error) {
      console.error('Failed to fetch endpoint detail:', error)
    }
  }

  const handleDelete = () => {
    setIsDeleteConfirmOpen(true)
  }

  const confirmDelete = async () => {
    setIsDeleting(true)
    try {
      await EndpointsApiService.deleteEndpoint(row.original.id)
      // 关闭确认对话框
      setIsDeleteConfirmOpen(false)
      // 刷新页面以更新列表
      window.location.reload()
    } catch (error) {
      console.error('Failed to delete endpoint:', error)
      alert('删除端点失败')
    } finally {
      setIsDeleting(false)
    }
  }

  const getStatusConfig = (status: string) => {
    const statusMap: Record<string, { label: string; variant: any }> = {
      Running: { label: '运行中', variant: 'default' },
      Stopped: { label: '已停用', variant: 'secondary' },
      Deleted: { label: '已删除', variant: 'destructive' },
    }
    
    return statusMap[status] || { label: status, variant: 'default' }
  }

  const getMethodBadgeVariant = (method: string) => {
    const methodMap: Record<string, string> = {
      GET: 'success',
      POST: 'default',
      PUT: 'warning',
      DELETE: 'destructive',
      PATCH: 'secondary',
    }
    
    return methodMap[method] || 'default'
  }

  const getMethodBadgeClass = (method: string) => {
    const methodClassMap: Record<string, string> = {
      GET: 'bg-green-100 text-green-800',
      POST: 'bg-purple-100 text-purple-800',
      PUT: 'bg-orange-100 text-orange-800',
      DELETE: 'bg-red-100 text-red-800',
      PATCH: 'bg-primary/10 text-primary',
    }
    
    return methodClassMap[method] || 'bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-200'
  }

  const toggleApiDetail = (index: number) => {
    setOpenApiDetails(prev => ({
      ...prev,
      [index]: !prev[index]
    }))
  }

  const copySwaggerJson = () => {
    if (endpointDetail?.swagger_spec) {
      navigator.clipboard.writeText(JSON.stringify(endpointDetail.swagger_spec, null, 2))
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  // 初始化第一个API详情为展开状态
  if (endpointDetail?.api_details && Object.keys(openApiDetails).length === 0) {
    const initialOpenState: Record<string, boolean> = {}
    endpointDetail.api_details.forEach((_: any, index: number) => {
      initialOpenState[index] = index === 0 // 默认展开第一个
    })
    setOpenApiDetails(initialOpenState)
  }

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant='ghost'
            className='flex h-8 w-8 p-0 data-[state=open]:bg-muted'
          >
            <MoreHorizontal className='h-4 w-4' />
            <span className='sr-only'>Open menu</span>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align='end' className='w-[160px]'>
          <DropdownMenuItem onClick={handleView}>
            查看
          </DropdownMenuItem>
          <DropdownMenuItem onClick={handleEdit}>
            编辑
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={handleDelete}>删除</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      {/* Delete Confirmation Dialog */}
      <Dialog open={isDeleteConfirmOpen} onOpenChange={setIsDeleteConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
            <DialogDescription>
              确定要删除端点 "{row.original.name}" 吗？此操作无法撤销。
            </DialogDescription>
          </DialogHeader>
          <div className='flex justify-end space-x-2'>
            <Button
              variant='outline'
              onClick={() => setIsDeleteConfirmOpen(false)}
              disabled={isDeleting}
            >
              取消
            </Button>
            <Button
              variant='destructive'
              onClick={confirmDelete}
              disabled={isDeleting}
            >
              {isDeleting ? '删除中...' : '删除'}
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* View Dialog */}
      <Dialog open={isViewOpen} onOpenChange={setIsViewOpen}>
        <DialogContent 
          className='max-w-4xl max-h-[90vh] overflow-y-auto'
          style={{ width: '90vw', maxWidth: 'none' }}
        >
          <DialogHeader>
            <DialogTitle>端点详情</DialogTitle>
          </DialogHeader>
          {endpointDetail && (
            <div className='space-y-6'>
              <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
                <div>
                  <span className='font-medium'>服务名称:</span>
                  <span className='ml-2'>{endpointDetail.name}</span>
                </div>
                <div>
                  <span className='font-medium'>基础URL:</span>
                  <span className='ml-2'>{endpointDetail.base_url || '-'}</span>
                </div>
                {/* <div>
                  <span className='font-medium'>当前状态:</span>
                  <Badge variant={getStatusConfig(endpointDetail.status).variant} className='ml-2'>
                    {getStatusConfig(endpointDetail.status).label}
                  </Badge>
                </div> */}
                <div>
                  <span className='font-medium'>创建时间:</span>
                  <span className='ml-2'>{new Date(endpointDetail.created_at).toLocaleString()}</span>
                </div>
                <div>
                  <span className='font-medium'>当前连接数:</span>
                  <span className='ml-2'>{endpointDetail.connection_count || 0}</span>
                </div>
              </div>

              <div>
                <h3 className='font-medium mb-3'>方法列表:</h3>
                <div className='space-y-2'>
                  {endpointDetail.api_details?.map((api: any, index: number) => (
                    <Collapsible 
                      key={index} 
                      open={openApiDetails[index] || false}
                      onOpenChange={() => toggleApiDetail(index)}
                    >
                      <CollapsibleTrigger asChild>
                        <div className='flex items-center justify-between cursor-pointer p-3 bg-gray-50 dark:bg-[#1f1f23] rounded-md hover:bg-gray-100 dark:hover:bg-[#303034]'>
                          <div className='flex items-center space-x-2'>
                            <Badge className={getMethodBadgeClass(api.method)}>
                              {api.method}
                            </Badge>
                            <span className='font-mono'>{api.path}</span>
                            {api.summary && <span className='text-muted-foreground'>- {api.summary}</span>}
                          </div>
                          {openApiDetails[index] ? (
                            <ChevronDown className='h-4 w-4' />
                          ) : (
                            <ChevronRight className='h-4 w-4' />
                          )}
                        </div>
                      </CollapsibleTrigger>
                      <CollapsibleContent className='mt-2 p-3 bg-gray-50 dark:bg-[#1f1f23] rounded-md border border-t-0 dark:border-[#3a3a3e]'>
                        <div className='space-y-2'>
                          <div>
                            <span className='font-medium'>方法:</span> {api.method}
                          </div>
                          <div>
                            <span className='font-medium'>路径:</span> {api.path}
                          </div>
                          {api.summary && (
                            <div>
                              <span className='font-medium'>摘要:</span> {api.summary}
                            </div>
                          )}
                          {api.description && (
                            <div>
                              <span className='font-medium'>描述:</span> {api.description}
                            </div>
                          )}
                          {api.path_params && api.path_params.length > 0 && (
                            <div>
                              <span className='font-medium'>路径参数:</span>
                              <ul className='list-disc list-inside ml-4'>
                                {api.path_params.map((param: any, paramIndex: number) => (
                                  <li key={paramIndex}>
                                    {param.name} ({param.param_type}) {param.required ? '(必填)' : '(可选)'}
                                    {param.description && ` - ${param.description}`}
                                  </li>
                                ))}
                              </ul>
                            </div>
                          )}
                          {api.query_params && api.query_params.length > 0 && (
                            <div>
                              <span className='font-medium'>查询参数:</span>
                              <ul className='list-disc list-inside ml-4'>
                                {api.query_params.map((param: any, paramIndex: number) => (
                                  <li key={paramIndex}>
                                    {param.name} ({param.param_type}) {param.required ? '(必填)' : '(可选)'}
                                    {param.description && ` - ${param.description}`}
                                  </li>
                                ))}
                              </ul>
                            </div>
                          )}
                          {api.request_body_schema && (
                            <div>
                              <span className='font-medium'>请求体:</span>
                              <pre className='mt-1 max-h-40 overflow-auto rounded bg-gray-100 dark:bg-[#1f1f23] p-2 text-xs border border-gray-200 dark:border-gray-700'>
                                {JSON.stringify(api.request_body_schema, null, 2)}
                              </pre>
                            </div>
                          )}
                          {api.response_schema && (
                            <div>
                              <span className='font-medium'>响应体:</span>
                              <pre className='mt-1 max-h-40 overflow-auto rounded bg-gray-100 dark:bg-[#1f1f23] p-2 text-xs'>
                                {JSON.stringify(api.response_schema, null, 2)}
                              </pre>
                            </div>
                          )}
                        </div>
                      </CollapsibleContent>
                    </Collapsible>
                  ))}
                </div>
              </div>

              <div>
                <div className='flex justify-between items-center mb-3'>
                  <h3 className='font-medium'>Swagger接口详情:</h3>
                  <Button
                    variant='outline'
                    size='sm'
                    onClick={copySwaggerJson}
                    className='flex items-center gap-2'
                  >
                    <Copy className='h-4 w-4' />
                    {copied ? '已复制' : '复制'}
                  </Button>
                </div>
                <JsonHighlighter className="max-h-96 overflow-auto rounded bg-gray-100 dark:bg-[#1f1f23] text-sm border border-gray-200 dark:border-gray-700">
                  {JSON.stringify(endpointDetail.swagger_spec, null, 2)}
                </JsonHighlighter>
              </div>
            </div>
          )}
          <div className='flex justify-end'>
            <Button onClick={() => setIsViewOpen(false)}>关闭</Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={isEditOpen} onOpenChange={setIsEditOpen}>
        <DialogContent 
          className='max-w-4xl max-h-[90vh] overflow-y-auto'
          style={{ width: '90vw', maxWidth: 'none' }}
        >
          <DialogHeader>
            <DialogTitle>编辑端点</DialogTitle>
          </DialogHeader>
          <div className='space-y-4'>
            <div>
              <label className='block text-sm font-medium'>服务名称</label>
              <input
                type='text'
                defaultValue={endpointDetail?.name || row.original.name}
                className='mt-1 w-full rounded border p-2'
              />
            </div>
            <div>
              <label className='block text-sm font-medium'>基础URL</label>
              <input
                type='text'
                defaultValue={endpointDetail?.base_url || ''}
                className='mt-1 w-full rounded border p-2'
                readOnly
              />
            </div>
            <div>
              <label className='block text-sm font-medium'>Swagger接口详情</label>
              <Textarea
                ref={swaggerTextareaRef}
                defaultValue={endpointDetail?.swagger_spec ? JSON.stringify(endpointDetail.swagger_spec, null, 2) : ''}
                className='mt-1 w-full rounded border p-2 min-h-[400px] font-mono text-sm resize-y'
                placeholder='请输入Swagger JSON内容'
              />
            </div>
          </div>
          <div className='flex justify-end space-x-2'>
            <Button variant='secondary' onClick={() => setIsEditOpen(false)}>
              关闭
            </Button>
            <Button>提交</Button>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}






























