import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useMemo, useState, useRef } from 'react'
import { Header } from '@/components/layout/header'
import { HeaderActions } from '@/components/layout/header-actions'
import { Main } from '@/components/layout/main'
import { Button } from '@/components/ui/button'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { ChevronLeftIcon, ChevronRightIcon } from '@radix-ui/react-icons'
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '@/components/ui/breadcrumb'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { toast } from 'sonner'
import { TableRagApiService } from '@/features/table-rag/data/api'
import type { TableSearchPagedRequest, EsSearchPagedResponse } from '@/features/table-rag/data/schema'
import { DataTablePagination } from '@/components/data-table/pagination'
import { Eye } from 'lucide-react'
import { useReactTable, getCoreRowModel, getPaginationRowModel } from '@tanstack/react-table'

function DatasetDetailPage() {
  const { datasetId } = Route.useParams()
  const [loading, setLoading] = useState(false)
  const [results, setResults] = useState<import('@/features/table-rag/data/schema').EsHit[]>([])
  const [total, setTotal] = useState<number>(0)
  const isFetchingRef = useRef(false)
  const lastKeyRef = useRef('')
  const [indexOpen, setIndexOpen] = useState(false)
  const [detailLoading, setDetailLoading] = useState(false)
  const [detail, setDetail] = useState<import('@/features/table-rag/data/schema').DatasetDetailResponse | null>(null)
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(20)
  // 任务列表相关状态
  const [tasksOpen, setTasksOpen] = useState(false)
  const [tasksLoading, setTasksLoading] = useState(false)
  const [tasks, setTasks] = useState<import('@/features/table-rag/data/schema').IngestTask[]>([])
  const [polling, setPolling] = useState<boolean>(false)
  const [taskDot, setTaskDot] = useState<'green' | 'yellow' | 'red' | 'gray'>('gray')
  const [latestUpdate, setLatestUpdate] = useState<string | null>(null)
  const headerRef = useRef(null)
  const bodyRef = useRef(null)

  // 同步表头和表体的水平滚动
  useEffect(() => {
    const header = headerRef.current
    const body = bodyRef.current
    if (!header || !body) return

    const syncScroll = (source, target) => {
      target.scrollLeft = source.scrollLeft
    }

    const onHeaderScroll = () => syncScroll(header, body)
    const onBodyScroll = () => syncScroll(body, header)

    header.addEventListener('scroll', onHeaderScroll)
    body.addEventListener('scroll', onBodyScroll)

    return () => {
      header.removeEventListener('scroll', onHeaderScroll)
      body.removeEventListener('scroll', onBodyScroll)
    }
  }, [])

  // 分页加载数据
  useEffect(() => {
    const key = datasetId || ''
    if (!key) {
      toast.warning('缺少数据集ID')
      return
    }
    // 防重复：开发环境 StrictMode 初次渲染会触发两次
    if (isFetchingRef.current && lastKeyRef.current === key) return
    isFetchingRef.current = true
    lastKeyRef.current = key

    const fetchData = async () => {
      try {
        setLoading(true)
        const res = await TableRagApiService.searchPaged({
          dataset_id: key,
          query: '',
          page: page,
          page_size: pageSize,
        })
        const hits = res.hits?.hits || []
        setResults(hits)
        setTotal(res.hits?.total?.value || hits.length)
      } catch (e: any) {
        toast.error(e?.message || '加载失败')
        setResults([])
        setTotal(0)
      } finally {
        setLoading(false)
        isFetchingRef.current = false
      }
    }
    fetchData()
  }, [datasetId, page, pageSize])

  // 加载数据集详情用于展示表名
  useEffect(() => {
    const key = datasetId || ''
    if (!key) return
    const fetchDetail = async () => {
      try {
        setDetailLoading(true)
        const d = await TableRagApiService.getDataset(key)
        setDetail(d)
      } catch (e: any) {
        // 仅用于展示表名，失败不阻塞页面
      } finally {
        setDetailLoading(false)
      }
    }
    fetchDetail()
  }, [datasetId])

  // 加载当前数据集的任务列表
  const loadTasks = async () => {
    if (!datasetId) return
    try {
      setTasksLoading(true)
      const list = await TableRagApiService.listTasks(datasetId, 1, 100)
      setTasks(list)
      const hasFailed = list.some((t) => t.status === 'Failed')
      const hasRunning = list.some((t) => ['Processing', 'Running', 'Queued', 'Pending'].includes(t.status))
      if (hasFailed) {
        setTaskDot('red')
      } else if (hasRunning) {
        setTaskDot('yellow')
      } else if (list.length > 0) {
        setTaskDot('green')
      } else {
        setTaskDot('gray')
      }
      const completed = list
        .filter((t) => t.status === 'Completed')
        .sort((a, b) => new Date(b.update_time).getTime() - new Date(a.update_time).getTime())
      setLatestUpdate(completed[0]?.update_time || null)
    } catch (e: any) {
      toast.error(e?.message || '任务列表加载失败')
      setTaskDot('gray')
    } finally {
      setTasksLoading(false)
    }
  }

  // 打开任务列表对话框时拉取一次数据
  useEffect(() => {
    if (tasksOpen) {
      loadTasks()
    }
  }, [tasksOpen])

  // 后台轮询任务状态，用于页头圆点与最新更新时间展示
  useEffect(() => {
    if (!datasetId) return
    setPolling(true)
    const timer = window.setInterval(() => {
      loadTasks()
    }, 60000)
    // 进入页面先拉一次
    loadTasks()
    return () => {
      window.clearInterval(timer)
      setPolling(false)
    }
  }, [datasetId])

  // 从 _source.row 中提取动态列
  const columns = useMemo(() => {
    const set = new Set<string>()
    // 优先使用 _source.row 的键，若不存在则回退到 _source 本身的可展示键
    results.forEach((hit) => {
      const src = (hit._source as any) || {}
      const row: Record<string, any> | undefined = src.row
      const keys = row ? Object.keys(row) : Object.keys(src).filter((k) => !['vector', 'page_content', 'metadata', 'task_id'].includes(k))
      keys.forEach((k) => set.add(k))
    })
    return Array.from(set)
  }, [results])

  // 创建表格数据适配器
  const tableData = useMemo(() => results, [results])
  const table = useReactTable({
    data: tableData,
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

  return (
    <>
      <Header fixed>
        <HeaderActions />
      </Header>
      <Main fixed fluid>
        <div className='sticky top-0 z-40 border-b bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='flex items-center justify-between py-3'>
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem>
                  <BreadcrumbLink href='/datasets'>知识库</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                  <BreadcrumbPage>知识库详情</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
            <div className='flex items-center gap-2'>
              <span
                aria-label='任务状态'
                title={
                  taskDot === 'green'
                    ? '所有数据集任务完成'
                    : taskDot === 'yellow'
                    ? '正在有任务运行中'
                    : taskDot === 'red'
                    ? '有失败的任务'
                    : '未知状态'
                }
                className={`inline-block h-2.5 w-2.5 rounded-full ${
                  taskDot === 'green'
                    ? 'bg-green-500'
                    : taskDot === 'yellow'
                    ? 'bg-yellow-500'
                    : taskDot === 'red'
                    ? 'bg-red-500'
                    : 'bg-muted'
                }`}
              />
              <div className='text-xs text-muted-foreground mr-2'>最近更新：{latestUpdate ? new Date(latestUpdate).toLocaleString() : '-'}</div>
              <Button
                variant='outline'
                onClick={() => setTasksOpen(true)}
                disabled={!datasetId}
              >
                <Eye className='h-4 w-4 mr-1' /> 任务列表
              </Button>
              <Button
                variant='outline'
                onClick={async () => {
                  setIndexOpen(true)
                  if (!detail && datasetId) {
                    try {
                      setDetailLoading(true)
                      const d = await TableRagApiService.getDataset(datasetId)
                      setDetail(d)
                    } catch (e: any) {
                      toast.error(e?.message || '加载索引配置失败')
                    } finally {
                      setDetailLoading(false)
                    }
                  }
                }}
              >查看索引</Button>
            </div>
          </div>
        </div>

        <div className='px-1 pb-20'>
          <Card className='border-none shadow-none'>
            <CardHeader>
              <CardTitle>数据表{detail?.table_name ? `：${detail.table_name}` : ''}</CardTitle>
            </CardHeader>
            <CardContent className='p-0'>
                <div className='border rounded overflow-auto h-[calc(100vh-270px)]'>
                  <Table className='min-w-[800px]'>
                    <TableHeader>
                      <TableRow>
                        {columns.map((c) => (
                          <TableHead key={c}>{c}</TableHead>
                        ))}
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {results.map((hit) => {
                        const src = (hit._source as any) || {}
                        const row: Record<string, any> | undefined = src.row
                        return (
                          <TableRow key={hit._id}>
                            {columns.map((c) => (
                              <TableCell key={c}>{row ? String(row[c] ?? '') : String(src[c] ?? '')}</TableCell>
                            ))}
                          </TableRow>
                        )})}
                      {results.length === 0 && (
                        <TableRow>
                          <TableCell colSpan={Math.max(columns.length, 1)} className='text-center text-sm text-muted-foreground py-8'>
                            {loading ? '加载中...' : '暂无数据'}
                          </TableCell>
                        </TableRow>
                      )}
                    </TableBody>
                  </Table>
                </div>
            </CardContent>
                
                {/* 分页控制 - 固定在Card底部 */}
                {total > 0 && (
                  <div className='border-t p-1 bg-background sticky bottom-0'>
                    <DataTablePagination table={table} />
                  </div>
                )}
          </Card>
        </div>
      </Main>

      {/* 索引配置只读Dialog */}
      <Dialog open={indexOpen} onOpenChange={(open) => {
        setIndexOpen(open)
        if (!open) {
          setPage(1)
        }
      }}>
        <DialogContent className='w-[70vw] max-w-none sm:max-w-[70vw] md:max-w-[70vw] lg:max-w-[70vw] xl:max-w-[70vw] max-h-[80vh] overflow-hidden'>
          <DialogHeader>
            <DialogTitle>索引配置</DialogTitle>
            <DialogDescription></DialogDescription>
          </DialogHeader>
          {detailLoading && (
            <div className='text-sm text-muted-foreground'>加载中...</div>
          )}
          {!detailLoading && detail && (
            <div className='space-y-4 overflow-hidden'>
              <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
                <div>
                  <div className='text-xs text-muted-foreground mb-1'>表名</div>
                  <div className='text-sm'>{detail.table_name}</div>
                </div>
              </div>

              {/* 统计信息 */}
              <Stats detail={detail} />

              <div className='space-y-2'>
                <div className='text-sm font-medium'>表结构</div>
                <div className='border rounded overflow-auto max-h-[55vh]'>
                  <Table className='min-w-[700px]'>
                    <TableHeader>
                      <TableRow>
                        <TableHead>字段名称</TableHead>
                        <TableHead>类型</TableHead>
                        <TableHead>参与检索</TableHead>
                        <TableHead>参与回复</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {paginate(detail.table_schema, page, pageSize).map((c) => {
                        const retrievalSet = toSet(detail.retrieval_column)
                        const replySet = toSet(detail.reply_column)
                        const isRetrieval = retrievalSet.has(c.name)
                        const isReply = replySet.has(c.name)
                        const rowClass = isRetrieval && isReply
                          ? 'bg-purple-50'
                          : isRetrieval
                          ? 'bg-amber-50'
                          : isReply
                          ? 'bg-emerald-50'
                          : ''
                        return (
                        <TableRow key={c.name} className={rowClass}>
                          <TableCell>{c.name}</TableCell>
                          <TableCell>{c.type}</TableCell>
                          <TableCell className='text-center'>
                            <div className='flex items-center justify-center'>
                              <input type='checkbox' disabled checked={!!c.searchable} className='h-4 w-4 cursor-not-allowed' />
                            </div>
                          </TableCell>
                          <TableCell className='text-center'>
                            <div className='flex items-center justify-center'>
                              <input type='checkbox' disabled checked={!!c.retrievable} className='h-4 w-4 cursor-not-allowed' />
                            </div>
                          </TableCell>
                        </TableRow>
                      )})}
                      {detail.table_schema.length === 0 && (
                        <TableRow>
                          <TableCell colSpan={4} className='text-center text-sm text-muted-foreground py-6'>无表结构数据</TableCell>
                        </TableRow>
                      )}
                    </TableBody>
                  </Table>
                </div>
                {/* 分页控制 */}
                {detail.table_schema.length > pageSize && (
                  <div className='flex items-center justify-between text-sm'>
                    <div>
                      第 {page} / {Math.max(1, Math.ceil(detail.table_schema.length / pageSize))} 页
                    </div>
                    <div className='flex items-center gap-2'>
                      <Button variant='outline' size='sm' onClick={() => setPage((p) => Math.max(1, p - 1))} disabled={page === 1}>上一页</Button>
                      <Button variant='outline' size='sm' onClick={() => setPage((p) => Math.min(Math.ceil(detail.table_schema.length / pageSize), p + 1))} disabled={page >= Math.ceil(detail.table_schema.length / pageSize)}>下一页</Button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* 任务列表对话框（与命中测试页一致）*/}
      <Dialog open={tasksOpen} onOpenChange={setTasksOpen}>
        <DialogContent className='w-[95vw] sm:max-w-[900px] max-h-[80vh] p-0'>
          <DialogHeader className='px-6 pt-6'>
            <DialogTitle>数据集任务列表</DialogTitle>
          </DialogHeader>
          <div className='px-6 pb-3 flex items-center justify-between'>
            <div className='text-sm'>当前数据集：{datasetId ? <Badge>{datasetId}</Badge> : '未选择'}</div>
            <div className='text-xs text-muted-foreground'>轮询：{polling ? '开启' : '关闭'}{tasksLoading ? '（加载中）' : ''}</div>
          </div>
          <div className='px-6 pb-6'>
            <div className='border rounded overflow-x-auto'>
              <div className='max-h-[60vh] overflow-auto'>
                <Table className='min-w-[720px]'>
                  <TableHeader>
                    <TableRow>
                      <TableHead>任务ID</TableHead>
                      <TableHead>文件ID</TableHead>
                      <TableHead>状态</TableHead>
                      <TableHead>更新时间</TableHead>
                      <TableHead>错误</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {tasks.map((t) => (
                      <TableRow key={t.id}>
                        <TableCell className='font-mono text-xs break-all'>{t.id}</TableCell>
                        <TableCell className='font-mono text-xs break-all'>{t.file_id}</TableCell>
                        <TableCell>
                          <Badge variant={t.status === 'Completed' ? 'default' : t.status === 'Failed' ? 'destructive' : 'outline'}>
                            {t.status}
                          </Badge>
                        </TableCell>
                        <TableCell className='text-xs'>{new Date(t.update_time).toLocaleString()}</TableCell>
                        <TableCell className='text-xs max-w-[280px] truncate' title={t.error || ''}>{t.error || '-'}</TableCell>
                      </TableRow>
                    ))}
                    {tasks.length === 0 && (
                      <TableRow>
                        <TableCell colSpan={5} className='text-center text-sm text-muted-foreground py-6'>暂无任务</TableCell>
                      </TableRow>
                    )}
                  </TableBody>
                </Table>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  )
}

function toSet(names: string | undefined) {
  return new Set((names || '').split(',').map((s) => s.trim()).filter(Boolean))
}

function paginate<T>(items: T[], page: number, pageSize: number): T[] {
  const start = (page - 1) * pageSize
  return items.slice(start, start + pageSize)
}

function Stats({ detail }: { detail: import('@/features/table-rag/data/schema').DatasetDetailResponse }) {
  const total = detail.table_schema.length
  const searchable = detail.table_schema.filter((c) => c.searchable).length
  const retrievable = detail.table_schema.filter((c) => c.retrievable).length
  return (
    <div className='grid grid-cols-2 md:grid-cols-3 gap-4'>
      <div className='rounded border p-3'>
        <div className='text-xs text-muted-foreground'>字段总数</div>
        <div className='text-sm font-medium'>{total}</div>
      </div>
      <div className='rounded border p-3'>
        <div className='text-xs text-muted-foreground'>可检索字段</div>
        <div className='text-sm font-medium'>{searchable}</div>
      </div>
      <div className='rounded border p-3'>
        <div className='text-xs text-muted-foreground'>可回复字段</div>
        <div className='text-sm font-medium'>{retrievable}</div>
      </div>
    </div>
  )
}

export const Route = createFileRoute('/_authenticated/datasets/$datasetId/view')({
  component: DatasetDetailPage,
})