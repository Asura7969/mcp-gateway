import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect, useMemo, useRef, useState } from 'react'
import { Header } from '@/components/layout/header'
import { HeaderActions } from '@/components/layout/header-actions'
import { Main } from '@/components/layout/main'
import { Card, CardContent, CardHeader, CardTitle, CardFooter } from '@/components/ui/card'
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '@/components/ui/breadcrumb'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { toast } from 'sonner'
import { UploadCloud, X, FileSpreadsheet, FileText as FileTextIcon } from 'lucide-react'
import { TableRagApiService } from '@/features/table-rag/data/api'
import type { DatasetDetailResponse, ColumnSchema } from '@/features/table-rag/data/schema'
import { uploadFiles, type UploadedFileMeta } from '@/features/files/api'

function EditDatasetPage() {
  const navigate = useNavigate()
  const { datasetId } = Route.useParams()

  const [loading, setLoading] = useState(true)
  const [dataset, setDataset] = useState<DatasetDetailResponse | null>(null)

  // 本地可编辑状态
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [similarity, setSimilarity] = useState<number[]>([0.2])
  const [maxResults, setMaxResults] = useState<number[]>([10])
  const [multiKey, setMultiKey] = useState(true)
  const [files, setFiles] = useState<File[]>([])
  const [uploaded, setUploaded] = useState<UploadedFileMeta[]>([])
  const [isUploading, setIsUploading] = useState(false)
  const [dragActive, setDragActive] = useState(false)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement | null>(null)

  const nameMax = 20
  const descMax = 1000

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true)
        const found = await TableRagApiService.getDataset(datasetId)
        if (!found?.id) {
          toast.error('未找到知识库')
          navigate({ to: '/datasets' })
          return
        }
        setDataset(found)
        // 预填充
        setName(found.name)
        setDescription(found.description || '')
        setSimilarity([found.similarity_threshold])
        setMaxResults([found.max_results])
      } catch (e) {
        toast.error('加载知识库失败')
      } finally {
        setLoading(false)
      }
    }
    load()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [datasetId])

  const handleCancel = () => navigate({ to: '/datasets' })
  const handleSave = async () => {
    if (!datasetId) return
    try {
      await TableRagApiService.updateDataset(datasetId, {
        name: name.trim(),
        description: description.trim() || undefined,
        similarity_threshold: similarity[0],
        max_results: maxResults[0],
      })
      toast.success('已保存修改')
      navigate({ to: '/datasets' })
    } catch (e: any) {
      toast.error(e?.message || '保存失败')
    }
  }

  // 上传区域行为与创建页保持一致
  const acceptTypes = {
    'text/csv': ['.csv'],
    'application/vnd.ms-excel': ['.xls'],
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': ['.xlsx'],
  }
  const acceptStr = Object.values(acceptTypes).flat().join(',')
  const maxSizeMB = 50

  const validateFile = (f: File) => {
    const ext = f.name.split('.').pop()?.toLowerCase() || ''
    const allowedExt = ['csv', 'xls', 'xlsx']
    if (!allowedExt.includes(ext)) {
      return `不支持的文件类型：${ext}. 仅支持 csv/xls/xlsx`
    }
    if (f.size > maxSizeMB * 1024 * 1024) {
      return `文件过大（>${maxSizeMB}MB）`
    }
    return null
  }

  const handleFiles = (list: FileList | null) => {
    if (!list || list.length === 0) return
    const next: File[] = []
    for (let i = 0; i < list.length; i++) {
      const f = list.item(i)!
      const err = validateFile(f)
      if (err) {
        setErrorMsg(err)
        toast.error(err)
        continue
      }
      next.push(f)
    }
    if (next.length > 0) {
      setErrorMsg(null)
      setFiles((prev) => [...prev, ...next])
    }
  }

  const onInputChange: React.ChangeEventHandler<HTMLInputElement> = (e) => {
    handleFiles(e.target.files)
  }

  const onDrop: React.DragEventHandler<HTMLDivElement> = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(false)
    handleFiles(e.dataTransfer.files)
  }

  const onDragOver: React.DragEventHandler<HTMLDivElement> = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(true)
  }

  const onDragLeave: React.DragEventHandler<HTMLDivElement> = (e) => {
    e.preventDefault()
    e.stopPropagation()
    setDragActive(false)
  }

  const triggerChoose = () => inputRef.current?.click()
  const removeFile = (idx: number) => setFiles((prev) => prev.filter((_, i) => i !== idx))
  const removeUploaded = (id: string) => setUploaded((prev) => prev.filter((u) => u.id !== id))
  const clearUploaded = () => setUploaded([])

  const doUpload = async () => {
    if (files.length === 0) {
      toast.warning('请选择至少一个文件')
      return
    }
    try {
      setIsUploading(true)
      const metas = await uploadFiles(files)
      setUploaded((prev) => [...prev, ...metas])
      toast.success(`上传成功：${metas.length}个文件`)
      setFiles([])
    } catch (e: any) {
      toast.error(e?.message || '上传失败')
    } finally {
      setIsUploading(false)
    }
  }

  const handleIngest = async () => {
    if (!datasetId) {
      toast.warning('缺少数据集ID')
      return
    }
    if (!dataset) {
      toast.warning('知识库未加载完成')
      return
    }
    if (uploaded.length === 0) {
      toast.warning('请先上传文件')
      return
    }
    try {
      // 1) 后端推断上传文件的列及类型
      const fileIds = uploaded.map(u => u.id)
      const previewCols = await TableRagApiService.previewSchema(fileIds)

      // 2) 与知识库schema按列名比对类型冲突
      const backendSchema = dataset.table_schema
      const byName = new Map<string, ColumnSchema>(backendSchema.map(c => [c.name, c]))
      const backendNames = new Set(backendSchema.map(c => c.name))
      const uploadedNames = new Set(previewCols.map(c => c.name))

      // 2.1) 缺少列（数据库中有，但文件中没有）
      const missingColumns = Array.from(backendNames).filter(n => !uploadedNames.has(n))
      // 2.2) 不支持的列（文件中有，但数据库中没有）
      const unsupportedColumns = Array.from(uploadedNames).filter(n => !backendNames.has(n))

      if (missingColumns.length > 0 || unsupportedColumns.length > 0) {
        const parts: string[] = []
        if (missingColumns.length > 0) {
          parts.push(`缺少列：${missingColumns.join(', ')}`)
        }
        if (unsupportedColumns.length > 0) {
          parts.push(`存在不支持的列：${unsupportedColumns.join(', ')}`)
        }
        toast.error(parts.join('；'))
        return
      }

      const conflicts: string[] = []
      for (const c of previewCols) {
        const base = byName.get(c.name)
        if (base && base.type !== c.type) {
          conflicts.push(`${c.name}: ${base.type} ≠ ${c.type}`)
        }
      }
      if (conflicts.length > 0) {
        toast.error(`列类型冲突：${conflicts.join('; ')}`)
        return
      }

      // 3) 创建异步导入任务（逐文件）
      for (const fid of fileIds) {
        try {
          const res = await TableRagApiService.ingestFile({ dataset_id: datasetId, file_id: fid })
          if (res.task_id) {
            toast.success(`异步任务已创建：${res.task_id}`)
          } else {
            toast.success('异步任务已创建')
          }
        } catch (e: any) {
          toast.error(e?.message || `创建异步任务失败（文件ID: ${fid}）`)
        }
      }
    } catch (e: any) {
      toast.error(e?.message || '预检失败或创建任务失败')
    }
  }

  const disabled = useMemo(() => loading || !dataset, [loading, dataset])

  return (
    <>
      <Header fixed>
        <HeaderActions />
      </Header>
      <Main fixed fluid>
        {/* 顶部粘性工具栏：面包屑 + 操作按钮 */}
        <div className='sticky top-0 z-40 border-b bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='flex items-center justify-between py-3'>
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem>
                  <BreadcrumbLink href='/datasets'>知识库</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                  <BreadcrumbPage>编辑知识库</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
        </div>

        {/* 中间滚动区域：表单内容 */}
        <div className='flex-1 overflow-auto'>
          <Card className='border-none rounded-none'>
            <CardHeader>
              <CardTitle>知识库基础信息</CardTitle>
            </CardHeader>
            <CardContent className='space-y-6 pb-24'>
            {/* 基础信息 */}
            <div className='space-y-4'>
              <Label htmlFor='kb-name'>知识库名称<span className='text-destructive'> *</span></Label>
              <div className='flex items-center gap-2'>
                <Input id='kb-name' value={name} maxLength={nameMax} onChange={(e) => setName(e.target.value)} disabled={disabled} />
                <span className='text-xs text-muted-foreground'>{name.length}/{nameMax}</span>
              </div>
              <Label htmlFor='kb-desc'>知识库描述</Label>
              <div className='flex items-start gap-2'>
                <Textarea id='kb-desc' value={description} maxLength={descMax} onChange={(e) => setDescription(e.target.value)} disabled={disabled} />
                <span className='text-xs text-muted-foreground'>{description.length}/{descMax}</span>
              </div>
            </div>

            {/* 数据类型与表名（只读） */}
            {dataset && (
              <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
                <div className='space-y-2'>
                  <Label>数据类型</Label>
                  <Input value={dataset.type} disabled />
                </div>
                <div className='space-y-2'>
                  <Label>表名</Label>
                  <Input value={dataset.table_name} disabled />
                </div>
              </div>
            )}

            {/* 知识库配置 */}
            <div className='space-y-4'>
              <div>
                <Label className='text-base'>配置参数</Label>
                <div className='text-xs text-muted-foreground'>检索相关参数调整</div>
              </div>

              <div className='space-y-2'>
                <div className='flex items-center justify-between'>
                  <Label>多字段关键词匹配</Label>
                  <Switch checked={multiKey} onCheckedChange={setMultiKey} disabled={disabled} />
                </div>
              </div>

              <div className='space-y-2'>
                <Label>向量模型</Label>
                <Alert>
                  <AlertDescription>
                    默认是 <code>text-embedding-v4</code>
                  </AlertDescription>
                </Alert>
              </div>

              <div className='space-y-2'>
                <div className='flex items-center justify-between'>
                  <Label>相似度阈值</Label>
                  <span className='text-xs text-muted-foreground'>{similarity[0].toFixed(2)}</span>
                </div>
                <Slider value={similarity} onValueChange={setSimilarity} min={0.01} max={1} step={0.01} disabled={disabled} />
              </div>

              <div className='space-y-2'>
                <div className='flex items-center justify-between'>
                  <Label>最大召回数量</Label>
                  <span className='text-xs text-muted-foreground'>{maxResults[0]}</span>
                </div>
                <Slider value={maxResults} onValueChange={setMaxResults} min={1} max={20} step={1} disabled={disabled} />
              </div>
            </div>

            {/* 表单结束 */}
            </CardContent>
            <CardFooter className='flex flex-col gap-2'>
              <div className='w-full border-t pt-4'>
                <div className='text-sm font-medium mb-2'>导入测试</div>
                <div className='space-y-3'>
                  <input
                    ref={inputRef}
                    type='file'
                    accept={acceptStr}
                    multiple
                    className='hidden'
                    onChange={onInputChange}
                  />
                  <div
                    onClick={triggerChoose}
                    onDrop={onDrop}
                    onDragOver={onDragOver}
                    onDragLeave={onDragLeave}
                    className={`flex flex-col items-center justify-center rounded-lg border border-dashed p-8 text-center transition-colors cursor-pointer ${dragActive ? 'bg-muted/50 border-muted-foreground' : 'hover:bg-muted/30'}`}
                  >
                    <UploadCloud className='h-8 w-8 text-primary mb-2' />
                    <div className='text-sm'>点击或拖拽上传 CSV/XLS/XLSX</div>
                    <div className='text-xs text-muted-foreground mt-1'>最大{maxSizeMB}MB · 支持 {acceptStr}</div>
                    <Button variant='secondary' className='mt-3'>选择文件</Button>
                  </div>

                  {errorMsg && (
                    <div className='text-sm text-destructive'>{errorMsg}</div>
                  )}

                  {files.length > 0 && (
                    <div className='space-y-2'>
                      {files.map((f, idx) => (
                        <div key={`${f.name}-${idx}`} className='flex items-center justify-between rounded-md border p-3'>
                          <div className='flex items-center gap-3'>
                            {f.name.toLowerCase().endsWith('.csv') ? (
                              <FileTextIcon className='h-5 w-5 text-primary' />
                            ) : (
                              <FileSpreadsheet className='h-5 w-5 text-primary' />
                            )}
                            <div className='text-sm'>
                              <div className='font-medium'>{f.name}</div>
                              <div className='text-xs text-muted-foreground'>{(f.size / 1024).toFixed(0)} KB</div>
                            </div>
                          </div>
                          <Button variant='ghost' size='icon' onClick={() => removeFile(idx)}>
                            <X className='h-4 w-4' />
                          </Button>
                        </div>
                      ))}
                      <div className='flex justify-end'>
                        <Button onClick={doUpload} disabled={isUploading}>
                          {isUploading ? '上传中...' : '上传选中文件'}
                        </Button>
                      </div>
                    </div>
                  )}

                  {uploaded.length > 0 && (
                    <div className='mt-4 rounded-md border p-3'>
                      <div className='flex items-center justify-between mb-2'>
                        <div className='text-sm font-medium'>已上传文件（{uploaded.length}）</div>
                        <Button variant='ghost' size='sm' onClick={clearUploaded}>
                          清空列表
                        </Button>
                      </div>
                      <div className='space-y-2'>
                        {uploaded.map((u, idx) => (
                          <div key={`${u.id}-${idx}`} className='flex items-center justify-between rounded-md border p-2'>
                            <div className='flex items-center gap-3'>
                              {u.type === 'csv' ? (
                                <FileTextIcon className='h-4 w-4 text-primary' />
                              ) : (
                                <FileSpreadsheet className='h-4 w-4 text-primary' />
                              )}
                              <div className='text-sm'>
                                <div className='font-medium'>{u.name || u.path}</div>
                                <div className='text-[11px] text-muted-foreground'>
                                  ID: {u.id}
                                  {typeof u.size === 'number' ? ` · ${(u.size / 1024).toFixed(0)} KB` : ''}
                                </div>
                              </div>
                            </div>
                            <Button variant='ghost' size='icon' onClick={() => removeUploaded(u.id)}>
                              <X className='h-4 w-4' />
                            </Button>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  <div className='grid grid-cols-1 md:grid-cols-[1fr_auto] gap-2'>
                    <div className='text-xs text-muted-foreground'>
                      开始导入前将自动校验文件列类型与知识库列类型是否冲突。
                    </div>
                    <Button variant='default' onClick={handleIngest} disabled={uploaded.length === 0 || !datasetId || loading}>开始导入</Button>
                  </div>
                  <div className='text-xs text-muted-foreground mt-1'>
                    成功创建后可在“知识库”页通过“任务列表”查看进度。
                  </div>
                </div>
              </div>
            </CardFooter>
          </Card>
        </div>

        {/* 底部粘性操作栏 */}
        <div className='sticky bottom-0 z-40 border-t bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='flex items-center justify-end gap-3 pt-2 pb-1'>
            <Button variant='outline' onClick={handleCancel} disabled={loading}>取消</Button>
            <Button onClick={handleSave} disabled={loading || !name.trim()}>保存</Button>
          </div>
        </div>
      </Main>
    </>
  )
}

export const Route = createFileRoute('/_authenticated/datasets/$datasetId/edit')({
  component: EditDatasetPage,
})