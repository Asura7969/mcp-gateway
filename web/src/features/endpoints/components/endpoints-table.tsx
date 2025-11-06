import { useState, useEffect, useMemo, useCallback } from 'react'
import {
  type ColumnFiltersState,
  type SortingState,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  getFacetedRowModel,
  getFacetedUniqueValues,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
  type Row,
  type Table,
} from '@tanstack/react-table'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Eye, Edit, Trash2, Settings, Copy, RefreshCw, Loader2, ChevronDown, ChevronRight } from 'lucide-react'
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
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { toast } from 'sonner'
import { type Endpoint } from '../data/schema'
import { EndpointsApiService } from '../data/api'
import { AxiosError } from 'axios'

// 懒加载的API详情组件
const LazyApiDetails = ({ api }: { api: any }) => {
  return (
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
        <div className={api.request_body_schema && api.response_schema && (
          (api.request_body_schema.type === 'array' && api.request_body_schema.items) ||
          (api.request_body_schema.properties && Object.keys(api.request_body_schema.properties).length > 0) ||
          (api.request_body_schema.type && api.request_body_schema.type !== 'object')
        ) && (
          (api.response_schema.type === 'array' && api.response_schema.items) ||
          (api.response_schema.properties && Object.keys(api.response_schema.properties).length > 0) ||
          (api.response_schema.type && api.response_schema.type !== 'object')
        ) ? "grid grid-cols-1 md:grid-cols-2 gap-4" : ""}>
          {api.request_body_schema && (
            (api.request_body_schema.type === 'array' && api.request_body_schema.items) ||
            (api.request_body_schema.properties && Object.keys(api.request_body_schema.properties).length > 0) ||
            (api.request_body_schema.type && api.request_body_schema.type !== 'object')
          ) && (
            <div>
              <ApiFieldDisplay schema={api.request_body_schema} title="请求体" />
            </div>
          )}
          {api.response_schema && (
            (api.response_schema.type === 'array' && api.response_schema.items) ||
            (api.response_schema.properties && Object.keys(api.response_schema.properties).length > 0) ||
            (api.response_schema.type && api.response_schema.type !== 'object')
          ) && (
            <div>
              <ApiFieldDisplay schema={api.response_schema} title="响应体" />
            </div>
          )}
        </div>
      )}
    </div>
  )
}

const JsonHighlighter = ({ children, className = "" }: { children: string; className?: string }) => {
  const [shouldRender, setShouldRender] = useState(false)

  // 缓存格式化的JSON字符串
  const formattedJson = useMemo(() => {
    try {
      // 如果字符串太长，先检查是否需要渲染
      if (children.length > 10000 && !shouldRender) {
        return children.substring(0, 1000) + '\n... (点击展开查看完整内容)'
      }
      return children
    } catch {
      return children
    }
  }, [children, shouldRender])

  // 懒加载处理
  const handleToggleRender = useCallback(() => {
    setShouldRender(prev => !prev)
  }, [])

  // 检查是否为大型JSON
  const isLargeJson = children.length > 10000

  if (isLargeJson && !shouldRender) {
    return (
      <div className={`${className} p-4`}>
        <div className="flex items-center justify-end mb-2">
          <Button
            variant="outline"
            size="sm"
            onClick={handleToggleRender}
            className="text-xs"
          >
            展开查看
          </Button>
        </div>
        <pre className="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap">
          {formattedJson}
        </pre>
      </div>
    )
  }

  return (
    <div className={className}>
      <SyntaxHighlighter
        language="json"
        style={github}
        customStyle={{
          background: 'transparent',
          padding: '1rem',
          fontSize: '0.875rem',
          lineHeight: '1.25rem',
        }}
        wrapLongLines={true}
        showLineNumbers={false}
      >
        {formattedJson}
      </SyntaxHighlighter>
    </div>
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
                       isSyncing,
                       loadingDetailId
                     }: {
  row: Row<Endpoint>,
  onView: (row: Row<Endpoint>) => void,
  onEdit: (row: Row<Endpoint>) => void,
  onDelete: (row: Row<Endpoint>) => void,
  onConfig: (row: Row<Endpoint>) => void,
  onSync: (row: Row<Endpoint>) => void,
  isSyncing: boolean,
  loadingDetailId: string | null
}) => {
  const isLoadingDetail = loadingDetailId === row.original.id

  return (
    <div className='flex items-center gap-2'>
      <Button
        variant='outline'
        size='sm'
        onClick={() => onView(row)}
        disabled={isLoadingDetail}
        className='h-8 w-8 p-0'
      >
        {isLoadingDetail ? (
          <Loader2 className='h-4 w-4 animate-spin' />
        ) : (
          <Eye className='h-4 w-4' />
        )}
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
  loading?: boolean
  table?: Table<any>
}

export function EndpointsTable({ data, onDataReload, loading = false, table }: DataTableProps) {
  // Local UI-only states
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({})
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])

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
  const [loadingDetailId, setLoadingDetailId] = useState<string | null>(null)

  // 添加缓存状态
  const [detailCache, setDetailCache] = useState<Map<string, any>>(new Map())

  // 处理操作函数
  const handleView = useCallback(async (row: Row<Endpoint>) => {
    const endpointId = row.original.id

    try {
      setLoadingDetailId(endpointId)
      setSelectedEndpoint(row.original)

      // 检查缓存
      if (detailCache.has(endpointId)) {
        const cachedDetail = detailCache.get(endpointId)
        setEndpointDetail(cachedDetail)

        // 初始化API详情状态
        if (cachedDetail?.api_details) {
          const initialOpenState: Record<string, boolean> = {}
          cachedDetail.api_details.forEach((_: any, index: number) => {
            initialOpenState[index] = index === 0
          })
          setOpenApiDetails(initialOpenState)
        }

        // 立即打开对话框（缓存数据）
        setIsViewOpen(true)
        setLoadingDetailId(null)
        return
      }

      // 获取详情数据
      const detail = await EndpointsApiService.getEndpointById(endpointId)

      // 预处理数据以提高渲染性能
      const processedDetail = {
        ...detail,
        // 预处理swagger_spec字符串
        swagger_spec_string: detail.swagger_spec
          ? JSON.stringify(detail.swagger_spec, null, 2)
          : null,
        // 预处理API详情
        processed_api_details: detail.api_details?.map((api: any, index: number) => ({
          ...api,
          id: `api_${index}`,
          // 预处理大型JSON字段
          request_schema_string: api.request_schema
            ? JSON.stringify(api.request_schema, null, 2)
            : null,
          response_schema_string: api.response_schema
            ? JSON.stringify(api.response_schema, null, 2)
            : null,
        })) || []
      }

      // 缓存处理后的数据
      setDetailCache(prev => new Map(prev).set(endpointId, processedDetail))
      setEndpointDetail(processedDetail)

      // 初始化API详情状态
      if (processedDetail.processed_api_details?.length > 0) {
        const initialOpenState: Record<string, boolean> = {}
        processedDetail.processed_api_details.forEach((_: any, index: number) => {
          initialOpenState[index] = index === 0 // 默认展开第一个
        })
        setOpenApiDetails(initialOpenState)
      }

      // 数据加载完成后再打开对话框
      setIsViewOpen(true)
    } catch (error) {
      console.error('Failed to fetch endpoint detail:', error)
      toast.error('获取端点详情失败，请稍后重试')
    } finally {
      setLoadingDetailId(null)
    }
  }, [detailCache])

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

  const handleDelete = (row: Row<Endpoint>) => {
    setSelectedEndpoint(row.original)
    setIsDeleteConfirmOpen(true)
  }

  const handleConfig = (row: Row<Endpoint>) => {
    setSelectedEndpoint(row.original)
    setIsConfigOpen(true)
  }

  const handleSync = async (row: Row<Endpoint>) => {
    setIsSyncing(true)
    try {
      await EndpointsApiService.syncEndpoint(row.original.name)
      toast.success('同步成功')
      onDataReload?.()
    } catch (error) {
      console.error('Failed to sync endpoint:', error)

      let errorMessage = '未知错误'
      if (error instanceof AxiosError) {
        // 尝试从响应中提取错误信息
        if (error.response?.data) {
          // 如果响应数据是字符串（纯文本错误）
          if (typeof error.response.data === 'string') {
            errorMessage = error.response.data
          } else {
            // 如果响应数据是对象（JSON错误）
            errorMessage = error.response.data.message ||
              error.response.data.error ||
              error.response.data.title ||
              error.message ||
              '未知错误'
          }
        } else {
          errorMessage = error.message || '未知错误'
        }
      } else if (error instanceof Error) {
        errorMessage = error.message
      }

      toast.error('同步失败', {
        description: errorMessage,
        duration: 10000,
        closeButton: true,
      })
    } finally {
      setIsSyncing(false)
    }
  }

  // 定义列
  const columns = [
    {
      accessorKey: 'name',
      header: 'service',
    },
    {
      accessorKey: 'description',
      header: 'description',
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
          loadingDetailId={loadingDetailId}
        />
      ),
    },
  ]

  // 如果没有传入 table 实例，则创建一个内部的（保持向后兼容）
  const internalTable = useReactTable({
    data,
    columns,
    state: {
      sorting,
      columnVisibility,
      columnFilters,
    },
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFacetedRowModel: getFacetedRowModel(),
    getFacetedUniqueValues: getFacetedUniqueValues(),
    onColumnFiltersChange: setColumnFilters,
  })

  const tableInstance = table || internalTable

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

      let errorMessage = '未知错误'
      if (error instanceof AxiosError) {
        // 尝试从响应中提取错误信息
        if (error.response?.data) {
          // 如果响应数据是字符串（纯文本错误）
          if (typeof error.response.data === 'string') {
            errorMessage = error.response.data
          } else {
            // 如果响应数据是对象（JSON错误）
            errorMessage = error.response.data.message ||
              error.response.data.error ||
              error.response.data.title ||
              error.message ||
              '未知错误'
          }
        } else {
          errorMessage = error.message || '未知错误'
        }
      } else if (error instanceof Error) {
        errorMessage = error.message
      }

      toast.error('更新端点失败', {
        description: errorMessage,
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

      let errorMessage = '未知错误'
      if (error instanceof AxiosError) {
        // 检查是否是纯文本错误响应
        if (typeof error.response?.data === 'string') {
          errorMessage = error.response.data
        } else {
          // 尝试从响应中提取错误信息
          errorMessage = error.response?.data?.message ||
            error.response?.data?.error ||
            error.response?.data?.title ||
            error.message ||
            '未知错误'
        }
      } else if (error instanceof Error) {
        errorMessage = error.message
      }

      toast.error('删除端点失败', {
        description: errorMessage,
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
      <div className='overflow-hidden rounded-md border'>
        <Table>
          <TableHeader>
            {tableInstance.getHeaderGroups().map((headerGroup) => (
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
            {loading ? (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className='h-24 text-center'
                >
                  <div className='flex items-center justify-center space-x-2'>
                    <Loader2 className='h-4 w-4 animate-spin' />
                    <span>加载中...</span>
                  </div>
                </TableCell>
              </TableRow>
            ) : tableInstance.getRowModel().rows?.length ? (
              tableInstance.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
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

      {/* View Dialog */}
      <Dialog open={isViewOpen} onOpenChange={setIsViewOpen}>
        <DialogContent
          className='max-w-4xl max-h-[90vh] overflow-y-auto'
          style={{ width: '90vw', maxWidth: 'none' }}
        >
          <DialogHeader>
            <DialogTitle>端点详情</DialogTitle>
          </DialogHeader>

          {/* Content - 现在只有在数据准备好后才打开对话框 */}
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
                        {openApiDetails[index] && (
                          <LazyApiDetails api={api} />
                        )}
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
                  {endpointDetail.swagger_spec_string || JSON.stringify(endpointDetail.swagger_spec, null, 2)}
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
          <div className='space-y-4' style={{ maxWidth: '85vw', width: '85vw' }}>
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
                className='mt-1 w-full rounded border p-2 min-h-[400px] max-h-[600px] font-mono text-sm resize-y break-words'
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
            <div className='space-y-4' style={{ maxWidth: '85vw', width: '85vw' }}>
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
            </div>
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

  // 处理数组类型的schema
  if (schema.type === 'array' && schema.items) {
    return (
      <div className="border border-gray-200 dark:border-gray-700 rounded-md p-3 bg-white dark:bg-[#1f1f23]">
        <span className='font-medium text-xs'>{title}:</span>
        <div className="mt-1">
          <div className="mb-2 text-xs text-gray-600 dark:text-gray-400">
            类型: 数组
          </div>
          <div className="ml-4">
            <span className="text-xs font-medium">数组元素:</span>
            <ApiFieldDisplay schema={schema.items} title="Item" />
          </div>
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