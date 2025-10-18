import { useState, useEffect } from 'react'
import {
  type ColumnFiltersState,
  type SortingState,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  getFacetedRowModel,
  getFacetedUniqueValues,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
  type Row,
} from '@tanstack/react-table'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Input } from '@/components/ui/input'
import { DataTablePagination } from '@/components/data-table'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Eye, Edit, Trash2, Settings, Copy, RefreshCw } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Textarea } from '@/components/ui/textarea'
import SyntaxHighlighter from 'react-syntax-highlighter'
import { github } from 'react-syntax-highlighter/dist/esm/styles/hljs'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { toast } from 'sonner'
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

// 定义操作列组件
const ActionsCell = ({ 
  row,
  onView,
  onEdit,
  onDelete,
  onConfig,
  onSync,
  isSyncing
}: {
  row: Row<Endpoint>,
  onView: (row: Row<Endpoint>) => void,
  onEdit: (row: Row<Endpoint>) => void,
  onDelete: (row: Row<Endpoint>) => void,
  onConfig: (row: Row<Endpoint>) => void,
  onSync: (row: Row<Endpoint>) => void,
  isSyncing: boolean
}) => {
  return (
    <div className='flex items-center gap-2'>
      <Button
        variant='outline'
        size='sm'
        onClick={() => onView(row)}
        className='h-8 w-8 p-0'
      >
        <Eye className='h-4 w-4' />
        <span className='sr-only'>查看</span>
      </Button>
      <Button
        variant='outline'
        size='sm'
        onClick={() => onEdit(row)}
        className='h-8 w-8 p-0'
      >
        <Edit className='h-4 w-4' />
        <span className='sr-only'>编辑</span>
      </Button>
      <Button
        variant='outline'
        size='sm'
        onClick={() => onConfig(row)}
        className='h-8 w-8 p-0'
      >
        <Settings className='h-4 w-4' />
        <span className='sr-only'>配置</span>
      </Button>
      <Button
        variant='outline'
        size='sm'
        onClick={() => onSync(row)}
        disabled={isSyncing}
        className='h-8 w-8 p-0'
      >
        <RefreshCw className={`h-4 w-4 ${isSyncing ? 'animate-spin' : ''}`} />
        <span className='sr-only'>同步</span>
      </Button>
      <Button
        variant='outline'
        size='sm'
        onClick={() => onDelete(row)}
        className='h-8 w-8 p-0'
      >
        <Trash2 className='h-4 w-4' />
        <span className='sr-only'>删除</span>
      </Button>
    </div>
  )
}

type DataTableProps = {
  data: Endpoint[]
  onDataReload?: () => void
}

export function EndpointsTable({ data, onDataReload }: DataTableProps) {
  // Local UI-only states
  const [rowSelection, setRowSelection] = useState({})
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({})
  const [globalFilter, setGlobalFilter] = useState('')
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [pagination, setPagination] = useState({
    pageIndex: 0,
    pageSize: 10,
  })

  // Dialog states
  const [isViewOpen, setIsViewOpen] = useState(false)
  const [isEditOpen, setIsEditOpen] = useState(false)
  const [isDeleteConfirmOpen, setIsDeleteConfirmOpen] = useState(false)
  const [isConfigOpen, setIsConfigOpen] = useState(false)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isSyncing, setIsSyncing] = useState(false)
  const [selectedEndpoint, setSelectedEndpoint] = useState<Endpoint | null>(null)
  const [endpointDetail, setEndpointDetail] = useState<any>(null)
  const [openApiDetails, setOpenApiDetails] = useState<Record<string, boolean>>({})
  const [copied, setCopied] = useState(false)

  // 添加ESC键关闭功能
  useEffect(() => {
    const handleEsc = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsViewOpen(false)
        setIsEditOpen(false)
        setIsConfigOpen(false)
        setIsDeleteConfirmOpen(false)
      }
    }

    window.addEventListener('keydown', handleEsc)
    return () => {
      window.removeEventListener('keydown', handleEsc)
    }
  }, [])

  // 定义列
  const columns = [
    {
      id: 'select',
      header: ({ table }: any) => (
        <input
          type="checkbox"
          checked={table.getIsAllPageRowsSelected()}
          onChange={table.toggleAllPageRowsSelected}
        />
      ),
      cell: ({ row }: any) => (
        <input
          type="checkbox"
          checked={row.getIsSelected()}
          onChange={row.toggleSelected}
        />
      ),
    },
    {
      accessorKey: 'name',
      header: 'service',
    },
    {
      accessorKey: 'description',
      header: 'description',
    },
    {
      accessorKey: 'connection_count',
      header: 'connection count',
    },
    {
      accessorKey: 'created_at',
      header: 'create time',
      cell: ({ row }: any) => {
        const date = new Date(row.getValue('created_at'))
        return <div>{date.toLocaleDateString()}</div>
      },
    },
    {
      id: 'actions',
      header: 'action',
      cell: ({ row }: { row: Row<Endpoint> }) => (
        <ActionsCell 
          row={row}
          onView={handleView}
          onEdit={handleEdit}
          onDelete={handleDelete}
          onConfig={handleConfig}
          onSync={handleSync}
          isSyncing={isSyncing}
        />
      ),
    },
  ]

  const table = useReactTable({
    data,
    columns,
    state: {
      sorting,
      columnVisibility,
      rowSelection,
      columnFilters,
      globalFilter,
      pagination,
    },
    enableRowSelection: true,
    onRowSelectionChange: setRowSelection,
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    globalFilterFn: (row, _columnId, filterValue) => {
      const name = String(row.getValue('name')).toLowerCase()
      const searchValue = String(filterValue).toLowerCase()

      return name.includes(searchValue)
    },
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFacetedRowModel: getFacetedRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
    onPaginationChange: setPagination,
    onGlobalFilterChange: setGlobalFilter,
    onColumnFiltersChange: setColumnFilters,
  })



  // 获取方法徽章样式
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

  // 切换API详情展开状态
  const toggleApiDetail = (index: number) => {
    setOpenApiDetails(prev => ({
      ...prev,
      [index]: !prev[index]
    }))
  }

  // 复制Swagger JSON
  const copySwaggerJson = () => {
    if (endpointDetail?.swagger_spec) {
      navigator.clipboard.writeText(JSON.stringify(endpointDetail.swagger_spec, null, 2))
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  // 处理查看操作
  const handleView = async (row: Row<Endpoint>) => {
    try {
      setSelectedEndpoint(row.original)
      const detail = await EndpointsApiService.getEndpointById(row.original.id)
      setEndpointDetail(detail)
      
      // 初始化第一个API详情为展开状态
      if (detail?.api_details) {
        const initialOpenState: Record<string, boolean> = {}
        detail.api_details.forEach((_: any, index: number) => {
          initialOpenState[index] = index === 0 // 默认展开第一个
        })
        setOpenApiDetails(initialOpenState)
      }
      
      setIsViewOpen(true)
    } catch (error) {
      console.error('Failed to fetch endpoint detail:', error)
    }
  }

  // 处理编辑操作
  const handleEdit = async (row: Row<Endpoint>) => {
    try {
      setSelectedEndpoint(row.original)
      const detail = await EndpointsApiService.getEndpointById(row.original.id)
      setEndpointDetail(detail)
      setIsEditOpen(true)
    } catch (error) {
      console.error('Failed to fetch endpoint detail:', error)
    }
  }

  // 处理删除操作
  const handleDelete = (row: Row<Endpoint>) => {
    setSelectedEndpoint(row.original)
    setIsDeleteConfirmOpen(true)
  }

  // 处理配置操作
  const handleConfig = (row: Row<Endpoint>) => {
    setSelectedEndpoint(row.original)
    setIsConfigOpen(true)
  }

  // 处理同步操作
   const handleSync = async (row: Row<Endpoint>) => {
     setIsSyncing(true)
     try {
       await EndpointsApiService.syncEndpoint(row.original.name)
       toast.success('同步成功')
       onDataReload?.()
     } catch (error) {
       console.error('Failed to sync endpoint:', error)
       toast.error('同步失败', {
         description: (error as Error).message || '未知错误',
         duration: 10000,
         closeButton: true,
       })
     } finally {
       setIsSyncing(false)
     }
   }

  // 处理更新端点操作
  const handleUpdateEndpoint = async () => {
    if (!selectedEndpoint || !endpointDetail) return
    
    try {
      // 获取表单中的输入值
      const nameInput = document.querySelector('input[type="text"]') as HTMLInputElement
      const swaggerTextarea = document.querySelector('textarea') as HTMLTextAreaElement
      
      const updateData = {
        name: nameInput?.value || endpointDetail.name,
        swagger_content: swaggerTextarea?.value || ''
      }
      
      await EndpointsApiService.updateEndpoint(selectedEndpoint.id, updateData)
      setIsEditOpen(false)
      // 调用父组件的重新加载函数而不是刷新整个页面
      if (onDataReload) {
        onDataReload()
      }
    } catch (error) {
      console.error('Failed to update endpoint:', error)
      toast.error('更新端点失败', {
        description: (error as Error).message || '未知错误',
        duration: 10000,
        closeButton: true,
      })
    }
  }

  // 确认删除
  const confirmDelete = async () => {
    if (!selectedEndpoint) return

    setIsDeleting(true)
    try {
      await EndpointsApiService.deleteEndpoint(selectedEndpoint.id)
      // 关闭确认对话框
      setIsDeleteConfirmOpen(false)
      // 重新加载数据而不是刷新整个页面
      onDataReload?.()
    } catch (error) {
      console.error('Failed to delete endpoint:', error)
      toast.error('删除端点失败', {
        description: (error as Error).message || '未知错误',
        duration: 10000,
        closeButton: true,
      })
    } finally {
      setIsDeleting(false)
    }
  }

  // 生成MCP配置JSON
  const generateMcpConfig = (endpoint: Endpoint) => {
    const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';
    return {
      mcpServers: {
        [endpoint.name]: {
          type: "sse",
          url: `${baseUrl}/${endpoint.id}/sse`
        }
      }
    }
  }

  // 生成Streamable配置JSON
  const generateStreamableConfig = (endpoint: Endpoint) => {
    const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';
    return {
      mcpServers: {
        [endpoint.name]: {
          type: "streamable_http",
          url: `${baseUrl}/stream/${endpoint.id}`
        }
      }
    }
  }

  return (
    <div className='space-y-4 max-sm:has-[div[role="toolbar"]]:mb-16'>
      <div className='flex flex-col sm:flex-row gap-4'>
        <Input
          placeholder='search by service name...'
          value={globalFilter ?? ''}
          onChange={(event) => setGlobalFilter(event.target.value)}
          className='max-w-sm'
        />
      </div>
      <div className='overflow-hidden rounded-md border'>
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id} colSpan={header.colSpan}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  )
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && 'selected'}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className='h-24 text-center'
                >
                  暂无数据
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
      <DataTablePagination table={table} />

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
            <div className='space-y-4' style={{ maxWidth: '85vw', width: '85vw' }}>
              <div className='grid grid-cols-1 md:grid-cols-2 gap-2'>
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
                <div className='space-y-2 text-xs'>
                  {endpointDetail.api_details?.map((api: any, index: number) => (
                    <Collapsible 
                      key={index} 
                      open={openApiDetails[index] || false}
                      onOpenChange={() => toggleApiDetail(index)}
                    >
                      <CollapsibleTrigger asChild>
                        <div className='flex items-center justify-between cursor-pointer p-3 bg-gray-50 dark:bg-[#1f1f23] rounded-md hover:bg-gray-100 dark:hover:bg-[#303034] text-xs'>
                          <div className='flex items-center space-x-2'>
                            <Badge className={getMethodBadgeClass(api.method)}>
                              {api.method}
                            </Badge>
                            <span className='font-mono text-xs'>{api.path}</span>
                            {api.summary && <span className='text-muted-foreground text-xs'>- {api.summary}</span>}
                          </div>
                          {openApiDetails[index] ? (
                            <ChevronDown className='h-4 w-4' />
                          ) : (
                            <ChevronRight className='h-4 w-4' />
                          )}
                        </div>
                      </CollapsibleTrigger>
                      <CollapsibleContent className='p-3 bg-gray-50 dark:bg-[#1f1f23] rounded-md dark:border-[#3a3a3e] text-xs'>
                        <div className='space-y-2 text-xs'>
                          <div>
                            <span className='font-medium text-xs'>路径:</span> <span className="text-xs">{api.path}</span>
                          </div>
                          {api.summary && (
                            <div>
                              <span className='font-medium text-xs'>摘要:</span> <span className="text-xs">{api.summary}</span>
                            </div>
                          )}
                          {api.description && (
                            <div>
                              <span className='font-medium text-xs'>描述:</span> <span className="text-xs">{api.description}</span>
                            </div>
                          )}
                          {api.path_params && api.path_params.length > 0 && (
                            <div>
                              <span className='font-medium text-xs'>路径参数:</span>
                              <ul className='list-disc list-inside ml-4 text-xs'>
                                {api.path_params.map((param: any, paramIndex: number) => (
                                  <li key={paramIndex} className="text-xs">
                                    {param.name} ({param.param_type}) {param.required ? '(必填)' : '(可选)'}
                                    {param.description && ` - ${param.description}`}
                                  </li>
                                ))}
                              </ul>
                            </div>
                          )}
                          {api.query_params && api.query_params.length > 0 && (
                            <div>
                              <span className='font-medium text-xs'>查询参数:</span>
                              <ul className='list-disc list-inside ml-4 text-xs'>
                                {api.query_params.map((param: any, paramIndex: number) => (
                                  <li key={paramIndex} className="text-xs">
                                    {param.name} ({param.param_type}) {param.required ? '(必填)' : '(可选)'}
                                    {param.description && ` - ${param.description}`}
                                  </li>
                                ))}
                              </ul>
                            </div>
                          )}
                          {(api.request_body_schema || api.response_schema) && (
                            <div className={api.request_body_schema && api.response_schema && Object.keys(api.request_body_schema).length > 0 && Object.keys(api.response_schema).length > 0 ? "grid grid-cols-1 md:grid-cols-2 gap-4" : ""}>
                              {api.request_body_schema && Object.keys(api.request_body_schema).length > 0 && (
                                <div>
                                  <ApiFieldDisplay schema={api.request_body_schema} title="请求体" />
                                </div>
                              )}
                              {api.response_schema && Object.keys(api.response_schema).length > 0 && (
                                <div>
                                  <ApiFieldDisplay schema={api.response_schema} title="响应体" />
                                </div>
                              )}
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
                defaultValue={endpointDetail?.name || selectedEndpoint?.name}
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
                defaultValue={endpointDetail?.swagger_spec ? JSON.stringify(endpointDetail.swagger_spec, null, 2) : ''}
                className='mt-1 w-full rounded border p-2 min-h-[400px] max-h-[600px] font-mono text-sm resize-y'
                placeholder='请输入Swagger JSON内容'
                rows={200}
              />
            </div>
          </div>
          <div className='flex justify-end space-x-2'>
            <Button variant='secondary' onClick={() => setIsEditOpen(false)}>
              关闭
            </Button>
            <Button onClick={handleUpdateEndpoint}>提交</Button>
          </div>
        </DialogContent>
      </Dialog>

      {/* Config Dialog */}
      <Dialog open={isConfigOpen} onOpenChange={setIsConfigOpen}>
        <DialogContent 
          className='max-w-4xl max-h-[90vh] overflow-y-auto'
          style={{ width: '90vw', maxWidth: 'none' }}
        >
          <DialogHeader>
            <DialogTitle>MCP客户端配置</DialogTitle>
            <DialogDescription>
              以下是用于连接到 {selectedEndpoint?.name} 端点的MCP客户端配置
            </DialogDescription>
          </DialogHeader>
          {selectedEndpoint && (
            <Tabs defaultValue="sse" className="w-full">
              <TabsList>
                <TabsTrigger value="sse">SSE</TabsTrigger>
                <TabsTrigger value="streamable">Streamable</TabsTrigger>
              </TabsList>
              <TabsContent value="sse" className="mt-4">
                <div>
                  <JsonHighlighter className="max-h-96 overflow-auto rounded bg-gray-100 dark:bg-[#1f1f23] text-sm border border-gray-200 dark:border-gray-700">
                    {JSON.stringify(generateMcpConfig(selectedEndpoint), null, 2)}
                  </JsonHighlighter>
                </div>
              </TabsContent>
              <TabsContent value="streamable" className="mt-4">
                <div>
                  <JsonHighlighter className="max-h-96 overflow-auto rounded bg-gray-100 dark:bg-[#1f1f23] text-sm border border-gray-200 dark:border-gray-700">
                    {JSON.stringify(generateStreamableConfig(selectedEndpoint), null, 2)}
                  </JsonHighlighter>
                </div>
              </TabsContent>
              <div className='mt-4 flex justify-end'>
                <Button onClick={() => setIsConfigOpen(false)}>关闭</Button>
              </div>
            </Tabs>
          )}
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <Dialog open={isDeleteConfirmOpen} onOpenChange={setIsDeleteConfirmOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
            <DialogDescription>
              确定要删除端点 "{selectedEndpoint?.name}" 吗？此操作无法撤销。
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
    </div>
  )
}

// 创建一个类似Apifox UI的JSON字段展示组件
function ApiFieldDisplay({ schema, title }: { schema: any; title: string }) {
  // 如果schema为空或不是对象，显示简单信息
  if (!schema || typeof schema !== 'object') {
    return (
      <div>
        <span className='font-medium text-xs'>{title}:</span>
        <div className="mt-1 p-2 bg-gray-100 dark:bg-[#1f1f23] rounded text-xs">
          {schema === null ? 'null' : typeof schema === 'object' ? 'object' : schema}
        </div>
      </div>
    );
  }

  // 渲染字段信息
  const renderField = (fieldSchema: any, fieldName: string, required: string[] = [], level = 0) => {
    // 获取字段类型
    const type = fieldSchema?.type || 'unknown';
    
    // 获取字段描述
    const description = fieldSchema?.description || '';
    
    // 检查是否为必填字段
    const isRequired = required.includes(fieldName);
    
    // 检查是否有子字段
    const hasProperties = fieldSchema?.properties && Object.keys(fieldSchema.properties).length > 0;
    
    // 检查是否为数组类型且有items
    const isArrayWithItems = type === 'array' && fieldSchema?.items;
    
    return (
      <div key={fieldName} className={`ml-${level * 4}`}>
        {hasProperties ? (
          // 对于有子字段的对象类型
          <Collapsible>
            <CollapsibleTrigger asChild>
              <div className="flex items-center justify-between cursor-pointer p-2 bg-gray-100 dark:bg-[#1f1f23] rounded hover:bg-gray-200 dark:hover:bg-[#303034] mb-1">
                <div className="flex items-center">
                  <span className="font-medium text-xs mr-2">{fieldName}</span>
                  <span className="text-primary text-xs mr-2">object</span>
                  {isRequired && <span className="text-red-500 text-xs mr-2">*</span>}
                  {description && <span className="text-gray-500 dark:text-gray-400 text-xs">- {description}</span>}
                </div>
                <ChevronDown className="h-4 w-4" />
              </div>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className="ml-4 border-l-2 border-gray-200 dark:border-[#3a3a3e] pl-2">
                {Object.entries(fieldSchema.properties).map(([subFieldName, subFieldSchema]: [string, any]) => 
                  renderField(subFieldSchema, subFieldName, fieldSchema.required || [], level + 1)
                )}
              </div>
            </CollapsibleContent>
          </Collapsible>
        ) : isArrayWithItems ? (
          // 对于数组类型
          <Collapsible>
            <CollapsibleTrigger asChild>
              <div className="flex items-center justify-between cursor-pointer p-2 bg-gray-100 dark:bg-[#1f1f23] rounded hover:bg-gray-200 dark:hover:bg-[#303034] mb-1">
                <div className="flex items-center">
                  <span className="font-medium text-xs mr-2">{fieldName}</span>
                  <span className="text-red-600 text-xs mr-2">array</span>
                  {isRequired && <span className="text-red-500 text-xs mr-2">*</span>}
                  {description && <span className="text-gray-500 dark:text-gray-400 text-xs">- {description}</span>}
                </div>
                <ChevronDown className="h-4 w-4" />
              </div>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className="ml-4 border-l-2 border-gray-200 dark:border-[#3a3a3e] pl-2">
                <div className="p-2 bg-gray-50 dark:bg-[#1f1f23] rounded mb-1">
                  <div className="flex items-center">
                    <span className="font-medium text-xs mr-2">items</span>
                    {fieldSchema.items.type && (
                      <span className={`text-xs mr-2 ${
                        fieldSchema.items.type === 'string' ? 'text-green-600' : 
                        fieldSchema.items.type === 'number' || fieldSchema.items.type === 'integer' ? 'text-purple-600' : 
                        fieldSchema.items.type === 'boolean' ? 'text-yellow-600' : 
                        fieldSchema.items.type === 'object' ? 'text-primary' : 
                        'text-gray-600'
                      }`}>
                        {fieldSchema.items.type}
                      </span>
                    )}
                  </div>
                </div>
                {/* 如果数组项是对象类型，显示其属性 */}
                {fieldSchema.items.type === 'object' && fieldSchema.items.properties && (
                  <div className="ml-4 border-l-2 border-gray-200 dark:border-[#3a3a3e] pl-2">
                    {Object.entries(fieldSchema.items.properties).map(([subFieldName, subFieldSchema]: [string, any]) => 
                      renderField(subFieldSchema, subFieldName, fieldSchema.items.required || [], level + 2)
                    )}
                  </div>
                )}
              </div>
            </CollapsibleContent>
          </Collapsible>
        ) : (
          // 对于基本类型字段
          <div className="p-2 bg-gray-50 dark:bg-[#1f1f23] rounded mb-1">
            <div className="flex items-center">
              <span className="font-medium text-xs mr-2">{fieldName}</span>
              <span className={`text-xs mr-2 ${
                type === 'string' ? 'text-green-600' : 
                type === 'number' || type === 'integer' ? 'text-purple-600' : 
                type === 'boolean' ? 'text-yellow-600' : 
                type === 'array' ? 'text-red-600' : 
                'text-gray-600'
              }`}>
                {type}
              </span>
              {isRequired && <span className="text-red-500 text-xs mr-2">*</span>}
              {description && <span className="text-gray-500 dark:text-gray-400 text-xs">- {description}</span>}
            </div>
          </div>
        )}
      </div>
    );
  };

  // 获取根对象的属性
  const properties = schema?.properties || {};
  const required = schema?.required || [];

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-md p-3 bg-white dark:bg-[#1f1f23]">
      <span className='font-medium text-xs'>{title}:</span>
      <div className="mt-1">
        {Object.entries(properties).map(([fieldName, fieldSchema]: [string, any]) => 
          renderField(fieldSchema, fieldName, required, 0)
        )}
        {Object.keys(properties).length === 0 && (
          <div className="p-2 bg-gray-100 rounded text-xs">
            无字段定义
          </div>
        )}
      </div>
    </div>
  );
}




























































































