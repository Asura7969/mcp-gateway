import { createFileRoute } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'
import { Header } from '@/components/layout/header'
import { HeaderActions } from '@/components/layout/header-actions'
import { Main } from '@/components/layout/main'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ResizableHandle, ResizablePanel, ResizablePanelGroup } from '@/components/ui/resizable'
import { Slider } from '@/components/ui/slider'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Textarea } from '@/components/ui/textarea'
import { Eye } from 'lucide-react'
import { toast } from 'sonner'
import { TableRagApiService } from '@/features/table-rag/data/api'
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '@/components/ui/breadcrumb'

export function DatasetHitTestPage() {
  const { datasetId } = Route.useParams()
  const [threshold, setThreshold] = useState<number[]>([0.3])
  const [maxResults, setMaxResults] = useState<number[]>([10])
  const [query, setQuery] = useState('')
  const [searching, setSearching] = useState(false)
  const [results, setResults] = useState<import('@/features/table-rag/data/schema').EsHit[]>([])
  const [total, setTotal] = useState<number>(0)
  
  // 任务列表相关状态
  const [tasksOpen, setTasksOpen] = useState(false)
  const [tasksLoading, setTasksLoading] = useState(false)
  const [tasks, setTasks] = useState<import('@/features/table-rag/data/schema').IngestTask[]>([])
  const [polling, setPolling] = useState<boolean>(false)
  const [taskDot, setTaskDot] = useState<'green' | 'yellow' | 'red' | 'gray'>('gray')

  useEffect(() => {
    if (!datasetId) {
      toast.warning('缺少数据集ID')
    }
  }, [datasetId])

  const doSearch = async () => {
    if (!datasetId) {
      toast.warning('缺少数据集ID')
      return
    }
    if (!query.trim()) {
      toast.warning('请输入检索关键词')
      return
    }
    try {
      setSearching(true)
      const res = await TableRagApiService.search({
        dataset_id: datasetId,
        query,
        max_results: maxResults[0],
        similarity_threshold: threshold[0],
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
      } else {
        setTaskDot('green')
      }
    } catch (e: any) {
      toast.error(e?.message || '任务列表加载失败')
      setTaskDot('gray')
    } finally {
      setTasksLoading(false)
    }
  }

  useEffect(() => {
    if (tasksOpen) {
      // 打开对话框时立即刷新一次
      loadTasks()
    }
  }, [tasksOpen])

  // 后台轮询任务状态用于页头圆点展示
  useEffect(() => {
    if (!datasetId) return
    setPolling(true)
    const timer = window.setInterval(() => {
      loadTasks()
    }, 60000) // 60秒轮询
    // 进入页面先拉一次
    loadTasks()
    return () => {
      window.clearInterval(timer)
      setPolling(false)
    }
  }, [datasetId])

  const queryPlaceholder = useMemo(() => '请输入文本', [])

  // 根据检索结果的 _source 提取所有字段，构建动态表头
  const dynamicColumns = useMemo(() => {
    const keys = new Set<string>()
    results.forEach((hit) => {
      const src: any = hit._source || {}
      Object.keys(src).forEach((k) => keys.add(k))
    })
    return Array.from(keys)
  }, [results])

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
                  <BreadcrumbPage>命中测试</BreadcrumbPage>
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
              <Button variant='outline' onClick={() => setTasksOpen(true)} disabled={!datasetId}>
                <Eye className='h-4 w-4 mr-1' /> 任务列表
              </Button>
            </div>
          </div>
        </div>
        <div className='flex-1 overflow-auto'>
        <ResizablePanelGroup direction='horizontal' className='h-full'>
          {/* 左侧配置面板 */}
          <ResizablePanel defaultSize={28} minSize={20} maxSize={40} className='p-2'>
            <Card className='h-full border-none shadow-none'>
              <CardHeader>
                <CardTitle>知识库配置调试</CardTitle>
              </CardHeader>
              <CardContent className='space-y-6'>
                <div>
                  <div className='flex items-center justify-between mb-2'>
                    <span className='text-sm'>相似度阈值</span>
                    <span className='text-xs text-muted-foreground'>{threshold[0].toFixed(2)}</span>
                  </div>
                  <Slider value={threshold} onValueChange={setThreshold} min={0.01} max={1} step={0.01} />
                </div>
                <div>
                  <div className='flex items-center justify-between mb-2'>
                    <span className='text-sm'>最大召回数量</span>
                    <span className='text-xs text-muted-foreground'>{maxResults[0]}</span>
                  </div>
                  <Slider value={maxResults} onValueChange={setMaxResults} min={1} max={50} step={1} />
                </div>
              </CardContent>
            </Card>
          </ResizablePanel>
          <ResizableHandle />
          {/* 右侧输入与结果区（垂直 Resizable 分隔） */}
          <ResizablePanel defaultSize={72} minSize={50} className='p-2'>
            <ResizablePanelGroup direction='vertical' className='h-full'>
              <ResizablePanel defaultSize={35} minSize={20}>
                <Card className='h-full border-none shadow-none'>
                  <CardHeader>
                    <CardTitle>输入</CardTitle>
                  </CardHeader>
                  <CardContent className='space-y-3'>
                    <Textarea placeholder={queryPlaceholder} value={query} onChange={(e) => setQuery(e.target.value)} />
                    <div className='flex justify-end'>
                      <Button onClick={doSearch} disabled={searching || !datasetId}>{searching ? '测试中...' : '测试'}</Button>
                    </div>
                  </CardContent>
                </Card>
              </ResizablePanel>
              <ResizableHandle />
              <ResizablePanel defaultSize={65} minSize={30}>
                <Card className='h-full border-none shadow-none'>
                  <CardHeader>
                    <CardTitle>召回结果 {total > 0 && <span className='text-muted-foreground text-sm'>（{total}）</span>}</CardTitle>
                  </CardHeader>
                  <CardContent className='h-full overflow-auto'>
                    <div className='overflow-x-auto'>
                      <Table className='min-w-[800px]'>
                        <TableHeader>
                          <TableRow>
                            <TableHead>评分</TableHead>
                            {dynamicColumns.map((col) => (
                              <TableHead key={col}>{col}</TableHead>
                            ))}
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {results.map((hit) => (
                            <TableRow key={hit._id}>
                              <TableCell>{hit._score?.toFixed(4)}</TableCell>
                              {dynamicColumns.map((col) => (
                                <TableCell key={col} className='max-w-[360px]'>
                                  {String((hit._source as any)?.[col] ?? '')}
                                </TableCell>
                              ))}
                            </TableRow>
                          ))}
                          {results.length === 0 && (
                            <TableRow>
                              <TableCell colSpan={dynamicColumns.length + 1} className='py-12'>
                                <div className='text-sm text-muted-foreground text-center'>暂无结果</div>
                              </TableCell>
                            </TableRow>
                          )}
                        </TableBody>
                      </Table>
                    </div>
                  </CardContent>
                </Card>
              </ResizablePanel>
            </ResizablePanelGroup>
          </ResizablePanel>
        </ResizablePanelGroup>
        </div>

        {/* 任务列表对话框 */}
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
      </Main>
    </>
  )
}

export const Route = createFileRoute('/_authenticated/datasets/$datasetId/hit')({
  component: DatasetHitTestPage,
})