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
import { Eye, Edit, Trash2, Settings, Copy } from 'lucide-react'
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
import { type Endpoint } from '../data/schema'
import { EndpointsApiService } from '../data/api'

// 定义操作列组件
const ActionsCell = ({ 
  row,
  onView,
  onEdit,
  onDelete,
  onConfig
}: {
  row: Row<Endpoint>,
  onView: (row: Row<Endpoint>) => void,
  onEdit: (row: Row<Endpoint>) => void,
  onDelete: (row: Row<Endpoint>) => void,
  onConfig: (row: Row<Endpoint>) => void
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
}

export function EndpointsTable({ data }: DataTableProps) {
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
      header: '服务名称',
    },
    {
      accessorKey: 'description',
      header: '描述',
    },
    {
      accessorKey: 'connection_count',
      header: '连接数',
    },
    {
      accessorKey: 'created_at',
      header: '创建时间',
      cell: ({ row }: any) => {
        const date = new Date(row.getValue('created_at'))
        return <div>{date.toLocaleDateString()}</div>
      },
    },
    {
      id: 'actions',
      header: '操作',
      cell: ({ row }: { row: Row<Endpoint> }) => (
        <ActionsCell 
          row={row}
          onView={handleView}
          onEdit={handleEdit}
          onDelete={handleDelete}
          onConfig={handleConfig}
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

  // 获取状态配置
  const getStatusConfig = (status: string) => {
    const statusMap: Record<string, { label: string; variant: any }> = {
      Running: { label: '运行中', variant: 'default' },
      Stopped: { label: '已停用', variant: 'secondary' },
      Deleted: { label: '已删除', variant: 'destructive' },
    }
    
    return statusMap[status] || { label: status, variant: 'default' }
  }

  // 获取方法徽章样式
  const getMethodBadgeClass = (method: string) => {
    const methodClassMap: Record<string, string> = {
      GET: 'bg-green-100 text-green-800',
      POST: 'bg-purple-100 text-purple-800',
      PUT: 'bg-orange-100 text-orange-800',
      DELETE: 'bg-red-100 text-red-800',
      PATCH: 'bg-blue-100 text-blue-800',
    }
    
    return methodClassMap[method] || 'bg-gray-100 text-gray-800'
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

  // 确认删除
  const confirmDelete = async () => {
    if (!selectedEndpoint) return

    setIsDeleting(true)
    try {
      await EndpointsApiService.deleteEndpoint(selectedEndpoint.id)
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

  // 生成MCP配置JSON
  const generateMcpConfig = (endpoint: Endpoint) => {
    return {
      mcpServers: {
        [endpoint.name]: {
          type: "sse",
          url: `http://localhost:3000/${endpoint.id}/sse`
        }
      }
    }
  }

  return (
    <div className='space-y-4 max-sm:has-[div[role="toolbar"]]:mb-16'>
      <div className='flex flex-col sm:flex-row gap-4'>
        <Input
          placeholder='按服务名称搜索...'
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
                <div>
                  <span className='font-medium'>当前状态:</span>
                  <Badge variant={getStatusConfig(endpointDetail.status).variant} className='ml-2'>
                    {getStatusConfig(endpointDetail.status).label}
                  </Badge>
                </div>
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
                        <div className='flex items-center justify-between cursor-pointer p-3 bg-gray-50 rounded-md hover:bg-gray-100'>
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
                      <CollapsibleContent className='mt-2 p-3 bg-gray-50 rounded-md border border-t-0'>
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
                              <pre className='mt-1 max-h-40 overflow-auto rounded bg-gray-100 p-2 text-xs'>
                                {JSON.stringify(api.request_body_schema, null, 2)}
                              </pre>
                            </div>
                          )}
                          {api.response_schema && (
                            <div>
                              <span className='font-medium'>响应体:</span>
                              <pre className='mt-1 max-h-40 overflow-auto rounded bg-gray-100 p-2 text-xs'>
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
                <SyntaxHighlighter language="json" style={github} className="max-h-96 overflow-auto rounded bg-gray-100 p-3 text-sm">
                  {JSON.stringify(endpointDetail.swagger_spec, null, 2)}
                </SyntaxHighlighter>
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
            <div className='space-y-4'>
              <div>
                <h3 className='font-medium mb-2'>SSE协议配置:</h3>
                <SyntaxHighlighter language="json" style={github} className="max-h-96 overflow-auto rounded bg-gray-100 p-3 text-sm">
                  {JSON.stringify(generateMcpConfig(selectedEndpoint), null, 2)}
                </SyntaxHighlighter>
              </div>
              <div className='mt-4 flex justify-end'>
                <Button onClick={() => setIsConfigOpen(false)}>关闭</Button>
              </div>
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




