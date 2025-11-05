import { useEffect, useMemo, useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { Header } from '@/components/layout/header'
import { Main } from '@/components/layout/main'
import { ProfileDropdown } from '@/components/profile-dropdown'
import { ThemeSwitch } from '@/components/theme-switch'
import { ConfigDrawer } from '@/components/config-drawer'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { toast } from 'sonner'
import { TableRagApiService } from './data/api'
import {
  type DatasetResponse,
  type EsHit,
  type CreateDatasetRequest,
} from './data/schema'
import { Database, Search as SearchIcon, Plus, FileText, Image as ImageIcon, Eye, MoreHorizontal, Copy } from 'lucide-react'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn, getPageNumbers } from '@/lib/utils'
import { useReactTable, getCoreRowModel, getPaginationRowModel } from '@tanstack/react-table'
import { DataTablePagination } from '@/components/data-table/pagination'

export function TableRagPage() {
  const navigate = useNavigate()
  const [datasets, setDatasets] = useState<DatasetResponse[]>([])
  const [loadingDatasets, setLoadingDatasets] = useState(false)
  const [selectedDatasetId, setSelectedDatasetId] = useState<string | null>(null)
  const [page, setPage] = useState<number>(1)
  const [pageSize, setPageSize] = useState<number>(20)
  const [total, setTotal] = useState<number>(0)
  const [hasNextPage, setHasNextPage] = useState<boolean>(false)

  const [creating, setCreating] = useState(false)
  const [newName, setNewName] = useState('')
  const [newTableName, setNewTableName] = useState('')
  const [newColumns, setNewColumns] = useState('')
  const [newDescription, setNewDescription] = useState('')
  const newSchema = useMemo(() => {
    // 将逗号分隔的列名转换为 ColumnSchema 数组，并默认 searchable=true
    return newColumns
      .split(',')
      .map((s) => s.trim())
      .filter(Boolean)
      .map((name) => ({
        name,
        type: 'string' as const,
        searchable: true,
        retrievable: true,
      }))
  }, [newColumns])

  const [query, setQuery] = useState('')
  const [threshold, setThreshold] = useState<number | ''>('')
  const [maxResults, setMaxResults] = useState<number>(10)
  const [searching, setSearching] = useState(false)
  const [results, setResults] = useState<EsHit[]>([])
  const [activeTab, setActiveTab] = useState<'all' | 'doc' | 'data' | 'image'>('all')

  // 创建表格实例用于 DataTablePagination
  const table = useReactTable({
    data: datasets,
    columns: [],
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    state: {
      pagination: {
        pageIndex: page - 1,
        pageSize: pageSize,
      },
    },
    pageCount: Math.ceil(total / pageSize),
    manualPagination: true,
    onPaginationChange: (updater) => {
      const newPagination = typeof updater === 'function' 
        ? updater(table.getState().pagination) 
        : updater
      
      // 更新页面大小
      if (newPagination.pageSize !== pageSize) {
        setPageSize(newPagination.pageSize)
        // 当页面大小改变时，重置到第一页
        setPage(1)
      }
      
      // 更新页面索引
      if (newPagination.pageIndex + 1 !== page) {
        setPage(newPagination.pageIndex + 1)
      }
    },
  })
  const [createOpen, setCreateOpen] = useState(false)
  // 移除任务列表相关状态
  // const [tasksOpen, setTasksOpen] = useState(false)
  // const [tasksLoading, setTasksLoading] = useState(false)
  // const [tasks, setTasks] = useState<import('./data/schema').IngestTask[]>([])
  // const [polling, setPolling] = useState<boolean>(false)

  const loadDatasets = async (p: number = page, ps: number = pageSize) => {
    try {
      setLoadingDatasets(true)
      const response = await TableRagApiService.listDatasets(p, ps)
      setDatasets(response.datasets)
      setTotal(response.pagination.total)
      setHasNextPage(response.pagination.page < response.pagination.total_pages)
      if (response.datasets.length > 0 && !selectedDatasetId) {
        setSelectedDatasetId(response.datasets[0].id)
      }
      // 开发模式下无数据时，注入一条本地Mock，便于预览样式
      if (response.datasets.length === 0 && import.meta.env.DEV) {
        const mock: DatasetResponse = {
          id: 'mock_kb_001',
          name: '表数据知识库',
          description: '联表混排',
          type: 'upload',
          table_name: 'kb_mock_table',
          similarity_threshold: 0.3,
          max_results: 10,
        }
        setDatasets([mock])
        setSelectedDatasetId(mock.id)
        setHasNextPage(false)
      }
    } catch (e) {
      toast.error('加载数据集失败')
      // 后端不可用或接口报错时，开发模式仍注入Mock，保证可视化效果
      if (import.meta.env.DEV) {
        const mock: DatasetResponse = {
          id: 'mock_kb_001',
          name: '表数据知识库',
          description: '联表混排',
          type: 'upload',
          table_name: 'kb_mock_table',
          similarity_threshold: 0.3,
          max_results: 10,
        }
        setDatasets([mock])
        setSelectedDatasetId(mock.id)
        setHasNextPage(false)
      }
    } finally {
      setLoadingDatasets(false)
    }
  }

  useEffect(() => {
    loadDatasets(page, pageSize)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, pageSize])

  const createDataset = async () => {
    if (!newName || !newTableName || newSchema.length === 0) {
      toast.warning('请填写名称、表名和至少一个列')
      return
    }
    try {
      setCreating(true)
      const payload: CreateDatasetRequest = {
        name: newName,
        description: newDescription || undefined,
        type: 'upload',
        table_name: newTableName,
        schema: newSchema,
        similarity_threshold: 0.3,
        max_results: 10,
      }
      const created = await TableRagApiService.createDataset(payload)
      toast.success('数据集创建成功')
      // 新建后跳转至第一页以看到最新数据
      setPage(1)
      setSelectedDatasetId(created.id)
      await loadDatasets(1, pageSize)
      setNewName('')
      setNewTableName('')
      setNewColumns('')
      setNewDescription('')
      setCreateOpen(false)
    } catch (e: any) {
      toast.error(e?.message || '创建失败')
    } finally {
      setCreating(false)
    }
  }

  const doSearch = async () => {
    if (!selectedDatasetId) {
      toast.warning('请先选择数据集')
      return
    }
    if (!query.trim()) {
      toast.warning('请输入检索关键词')
      return
    }
    try {
      setSearching(true)
      const res = await TableRagApiService.search({
        dataset_id: selectedDatasetId,
        query,
        max_results: maxResults,
        similarity_threshold: typeof threshold === 'number' ? threshold : undefined,
      })
      const hits = res.hits?.hits || []
      setResults(hits)
      setTotal(res.hits?.total?.value || hits.length)
    } catch (e: any) {
      toast.error(e?.message || '检索失败')
      setResults([])
      setTotal(0)
    } finally {
      setSearching(false)
    }
  }

  const filtered = useMemo(() => {
    let data = datasets
    if (activeTab === 'data') {
      data = datasets.filter(() => true) // 目前全部视为“数据”类型
    }
    if (activeTab === 'doc' || activeTab === 'image') {
      data = [] // 暂无文档/图片类型，返回空列表
    }
    if (query.trim()) {
      const q = query.trim().toLowerCase()
      data = data.filter(
        (d) => d.name.toLowerCase().includes(q) || (d.description || '').toLowerCase().includes(q)
      )
    }
    return data
  }, [datasets, activeTab, query])

  const kbCount = filtered.length

  const handleViewDetails = (d: DatasetResponse) => {
    navigate({ to: '/datasets/$datasetId/view', params: { datasetId: d.id } })
  }
  const handleHitTest = (d: DatasetResponse) => {
    navigate({ to: '/datasets/$datasetId/hit', params: { datasetId: d.id } })
  }
  const handleCopyId = async (id: string) => {
    try {
      await navigator.clipboard.writeText(id)
      toast.success('已复制ID')
    } catch {
      toast.error('复制失败')
    }
  }

  const handleEdit = (d: DatasetResponse) => {
    navigate({ to: '/datasets/$datasetId/edit', params: { datasetId: d.id } })
  }
  const handleDelete = (d: DatasetResponse) => {
    setDatasets((prev) => prev.filter((x) => x.id !== d.id))
    if (selectedDatasetId === d.id) {
      const next = datasets.find((x) => x.id !== d.id)
      setSelectedDatasetId(next ? next.id : null)
    }
    toast.success('已删除')
  }

  // 移除任务列表相关函数
  // const loadTasks = async () => {
  //   if (!selectedDatasetId) return
  //   try {
  //     setTasksLoading(true)
  //     const list = await TableRagApiService.listTasks(selectedDatasetId, 1, 100)
  //     setTasks(list)
  //   } catch (e: any) {
  //     toast.error(e?.message || '任务列表加载失败')
  //   } finally {
  //     setTasksLoading(false)
  //   }
  // }

  // useEffect(() => {
  //   let timer: number | null = null
  //   if (tasksOpen) {
  //     loadTasks()
  //     timer = window.setInterval(() => {
  //       loadTasks()
  //     }, 3000)
  //     setPolling(true)
  //   }
  //   return () => {
  //     if (timer) {
  //       window.clearInterval(timer)
  //     }
  //     setPolling(false)
  //   }
  // }, [tasksOpen, selectedDatasetId])

  return (
    <>
      <Header>
        <div className='ms-auto flex items-center space-x-4'>
          <ThemeSwitch />
          <ConfigDrawer />
          <ProfileDropdown />
        </div>
      </Header>
      <Main fixed fluid>
        <div className='flex-1 overflow-auto'>
        <div className='mb-4 flex items-center justify-between'>
          <div className='flex items-center gap-2'>
            <h2 className='text-xl font-semibold'>知识库</h2>
            <Badge variant='outline'>{kbCount}</Badge>
          </div>
          <div className='flex items-center gap-2'>
            <div className='relative'>
              <SearchIcon className='absolute left-2 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground' />
              <Input
                placeholder='搜索'
                className='pl-8 w-64'
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
            </div>
            <Button onClick={() => navigate({ to: '/datasets/create', search: { step: 1 as any } })}>
              <Plus className='h-4 w-4 mr-1' /> 创建知识库
            </Button>
            {/* 移除任务列表按钮 */}
            {/* <Button variant='outline' onClick={() => setTasksOpen(true)} disabled={!selectedDatasetId}>
              <Eye className='h-4 w-4 mr-1' /> 任务列表
            </Button> */}
          </div>
        </div>

        <Tabs value={activeTab} onValueChange={(v: any) => setActiveTab(v)} className='min-h-[600px]'>
          <TabsList>
            <TabsTrigger value='all'>全部</TabsTrigger>
            <TabsTrigger value='doc'>文档</TabsTrigger>
            <TabsTrigger value='data'>数据</TabsTrigger>
            <TabsTrigger value='image'>图片</TabsTrigger>
          </TabsList>

          <TabsContent value='all'>
            <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4'>
              {loadingDatasets ? (
                <div className='text-sm'>加载中...</div>
              ) : (
                filtered.map((d) => (
                  <Card
                    key={d.id}
                    className='group relative transition-all duration-200 cursor-pointer border border-primary/30 hover:border-primary hover:shadow-md hover:ring-1 hover:ring-primary/20'
                    onClick={() => setSelectedDatasetId(d.id)}
                  >
                    <CardContent className='pt-0'>
                      <div className='flex items-start gap-3'>
                        <div className='h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center'>
                          <Database className='h-6 w-6 text-primary' />
                        </div>
                        <div className='flex-1'>
                          <div className='flex items-center justify-between'>
                            <div className='font-medium'>{d.name}</div>
                          </div>
                           <div className='mt-3 space-y-2 pb-2'>
                            <div className='flex items-center gap-2 text-sm'>
                              <span className='text-muted-foreground'>描述</span>
                              <span>{d.description || '暂无描述'}</span>
                            </div>
                            <div className='flex items-center gap-2 text-sm'>
                              <span className='text-muted-foreground'>ID</span>
                              <span className='font-mono'>{d.id.slice(0, 10)}</span>
                              <Button variant='ghost' size='icon' onClick={() => handleCopyId(d.id)}>
                                <Copy className='h-4 w-4' />
                              </Button>
                            </div>
                            <div className='text-xs mt-1'>表名 <span className='text-muted-foreground'>{d.table_name}</span></div>
                          </div>
                          {/* 悬停操作区（卡片底部） */}
                          <div className='absolute left-3 right-3 bottom-3 z-10 opacity-0 group-hover:opacity-100 transition-opacity bg-background/90 backdrop-blur-sm rounded-md border p-1.5 flex gap-2 justify-center'>
                            <Button size='sm' onClick={() => handleViewDetails(d)}>
                              查看详情
                            </Button>
                            <Button variant='outline' size='sm' onClick={() => handleHitTest(d)}>命中测试</Button>
                            <DropdownMenu modal={false}>
                              <DropdownMenuTrigger asChild>
                                <Button variant='outline' size='sm' className='px-2'>
                                  <MoreHorizontal className='h-3 w-3' />
                                </Button>
                              </DropdownMenuTrigger>
                              <DropdownMenuContent align='end'>
                                <DropdownMenuItem onClick={() => handleEdit(d)}>编辑</DropdownMenuItem>
                                <DropdownMenuItem variant='destructive' onClick={() => handleDelete(d)}>删除</DropdownMenuItem>
                              </DropdownMenuContent>
                            </DropdownMenu>
                          </div>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))
              )}
              {(!loadingDatasets && filtered.length === 0) && (
                <div className='text-sm text-muted-foreground'>暂无知识库</div>
              )}
            </div>
            
          </TabsContent>
          <TabsContent value='doc'>
            <div className='text-sm text-muted-foreground'>暂无文档类型</div>
          </TabsContent>
          <TabsContent value='data'>
            <div className='grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4'>
              {filtered.map((d) => (
                <Card
                  key={d.id}
                  className='group relative transition-all duration-200 cursor-pointer border border-primary/30 hover:border-primary hover:shadow-md hover:ring-1 hover:ring-primary/20'
                  onClick={() => setSelectedDatasetId(d.id)}
                >
                  <CardContent className='pt-0'>
                    <div className='flex items-start gap-3'>
                      <div className='h-10 w-10 rounded-lg bg-primary/10 flex items-center justify-center'>
                        <Database className='h-6 w-6 text-primary' />
                      </div>
                      <div className='flex-1'>
                        <div className='font-medium'>{d.name}</div>
                         <div className='mt-3 space-y-2 pb-2'>
                          <div className='flex items-center gap-2 text-sm'>
                            <span className='text-muted-foreground'>描述</span>
                            <span>{d.description || '暂无描述'}</span>
                          </div>
                          <div className='flex items-center gap-2 text-sm'>
                            <span className='text-muted-foreground'>ID</span>
                            <span className='font-mono'>{d.id.slice(0, 10)}</span>
                            <Button variant='ghost' size='icon' onClick={() => handleCopyId(d.id)}>
                              <Copy className='h-4 w-4' />
                            </Button>
                          </div>
                          <div className='text-xs mt-1'>表名 <span className='text-muted-foreground'>{d.table_name}</span></div>
                        </div>
                        {/* 悬停操作区（卡片底部） */}
                        <div className='absolute left-3 right-3 bottom-3 z-10 opacity-0 group-hover:opacity-100 transition-opacity bg-background/90 backdrop-blur-sm rounded-md border p-1.5 flex gap-2 justify-center'>
                          <Button size='sm' onClick={() => handleViewDetails(d)}>
                            <Eye className='h-3 w-3 mr-1' /> 查看详情
                          </Button>
                          <Button variant='outline' size='sm' onClick={() => handleHitTest(d)}>命中测试</Button>
                          <DropdownMenu modal={false}>
                            <DropdownMenuTrigger asChild>
                              <Button variant='outline' size='sm' className='px-2'>
                                <MoreHorizontal className='h-3 w-3' />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align='end'>
                              <DropdownMenuItem onClick={() => handleEdit(d)}>编辑</DropdownMenuItem>
                              <DropdownMenuItem variant='destructive' onClick={() => handleDelete(d)}>删除</DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
            
          </TabsContent>
          <TabsContent value='image'>
            <div className='text-sm text-muted-foreground'>暂无图片类型</div>
          </TabsContent>

          <TabsContent value='search'>
            <div className='grid grid-cols-1 lg:grid-cols-3 gap-6'>
              <Card>
                <CardHeader>
                  <CardTitle>检索条件</CardTitle>
                </CardHeader>
                <CardContent className='space-y-3'>
                  <div className='text-sm'>当前数据集：{selectedDatasetId ? <Badge>{selectedDatasetId}</Badge> : '未选择'}</div>
                  <Input placeholder='关键词' value={query} onChange={(e) => setQuery(e.target.value)} />
                  <Input
                    placeholder='相似度阈值（可选，例如 0.3）'
                    value={threshold}
                    onChange={(e) => {
                      const v = e.target.value
                      setThreshold(v === '' ? '' : Number(v))
                    }}
                  />
                  <Input
                    placeholder='最大结果数'
                    value={maxResults}
                    onChange={(e) => setMaxResults(Number(e.target.value) || 10)}
                  />
                  <Button onClick={doSearch} disabled={searching || !selectedDatasetId}>
                    {searching ? '检索中...' : '检索'}
                  </Button>
                </CardContent>
              </Card>

              <Card className='lg:col-span-2'>
                <CardHeader>
                  <CardTitle>检索结果 {total > 0 && <span className='text-muted-foreground text-sm'>（{total}）</span>}</CardTitle>
                </CardHeader>
                <CardContent>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>评分</TableHead>
                        <TableHead>内容片段</TableHead>
                        <TableHead>行数据</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {results.map((hit) => (
                        <TableRow key={hit._id}>
                          <TableCell>{hit._score?.toFixed(4)}</TableCell>
                          <TableCell className='max-w-[420px] truncate'>{hit._source?.page_content}</TableCell>
                          <TableCell>
                            <div className='text-xs text-muted-foreground'>
                              {Object.entries(hit._source?.row || {}).map(([k, v]) => (
                                <div key={k}>
                                  <span className='font-medium'>{k}:</span> {String(v)}
                                </div>
                              ))}
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                      {results.length === 0 && (
                        <TableRow>
                          <TableCell colSpan={3}>
                            <div className='text-sm text-muted-foreground'>暂无结果</div>
                          </TableCell>
                        </TableRow>
                      )}
                    </TableBody>
                  </Table>
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        </Tabs>
        </div>

        {/* 底部粘性分页栏：使用 DataTablePagination 组件 */}
        <div className='sticky bottom-0 z-40 border-t bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='p-1'>
            <DataTablePagination table={table} />
          </div>
        </div>


      </Main>
    </>
  )
}