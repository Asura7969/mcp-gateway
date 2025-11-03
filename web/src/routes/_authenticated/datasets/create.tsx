import { createFileRoute, useNavigate, Link } from '@tanstack/react-router'
import { useMemo, useRef, useState, useEffect } from 'react'
import { Header } from '@/components/layout/header'
import { HeaderActions } from '@/components/layout/header-actions'
import { Main } from '@/components/layout/main'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Label } from '@/components/ui/label'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Button } from '@/components/ui/button'
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from '@/components/ui/breadcrumb'
import { FileText, Database, Image as ImageIcon, UploadCloud, X, FileSpreadsheet, FileText as FileTextIcon } from 'lucide-react'
import { toast } from 'sonner'
import { uploadFiles, type UploadedFileMeta } from '@/features/files/api'
import { TableRagApiService } from '@/features/table-rag/data/api'
import type { ColumnSchema as BackendColumnSchema } from '@/features/table-rag/data/schema'
import { Switch } from '@/components/ui/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Slider } from '@/components/ui/slider'
import { Alert, AlertDescription } from '@/components/ui/alert'

function CreateDatasetPage() {
  const navigate = useNavigate()
  const { step } = Route.useSearch()
  const currentStep = useMemo(() => {
    const s = Number(step)
    if (s === 2) return 2
    if (s === 3) return 3
    return 1
  }, [step])
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [kbType, setKbType] = useState<'doc' | 'data' | 'image'>('data')
  const [sourceType, setSourceType] = useState<'upload' | 'select' | 'related'>('upload')
  const [tableName, setTableName] = useState('')

  // 上传相关状态（多文件）
  const [files, setFiles] = useState<File[]>([])
  const [uploaded, setUploaded] = useState<UploadedFileMeta[]>([])
  const [isUploading, setIsUploading] = useState(false)
  const [dragActive, setDragActive] = useState(false)
  const [errorMsg, setErrorMsg] = useState<string | null>(null)
  const inputRef = useRef<HTMLInputElement | null>(null)

  const nameMax = 20
  const descMax = 1000

  const [checkingSchema, setCheckingSchema] = useState(false)

  const getSchemaNames = async (fileId: string): Promise<string[]> => {
    const cols = await TableRagApiService.previewSchema([fileId])
    return cols.map((c) => c.name.trim())
  }

  const checkUploadedSchemaConsistency = async (): Promise<boolean> => {
    if (uploaded.length <= 1) return true
    // 一次性请求所有文件的schema，避免多次向后端请求
    const fileIds = uploaded.map(u => u.id)
    const allSchemas = await TableRagApiService.previewSchema(fileIds)
    const baseNames = new Set(allSchemas.map(c => c.name.trim()))
    // 如果后端返回的是合并视图，则以该集合作为一致性基准
    // 若未来需要按文件区分返回，则可以调整为分文件比对
    // 当前简单判断：存在即一致
    return baseNames.size > 0
  }

  const handleNext = async () => {
    if (currentStep === 1) {
      if (!name.trim()) {
        toast.warning('请填写知识库名称')
        return
      }
      navigate({ to: '/datasets/create', search: { step: 2 as any } })
      return
    }
    if (currentStep === 2) {
      // 简单校验：上传模式下需要有已上传文件
      if (sourceType === 'upload' && uploaded.length === 0) {
        toast.warning('请先上传至少一个文件')
        return
      }
      // 必填校验：数据表名称
      if (!tableName.trim()) {
        toast.warning('请填写数据表名称')
        return
      }
      if (sourceType === 'upload') {
        try {
          setCheckingSchema(true)
          const ok = await checkUploadedSchemaConsistency()
          if (!ok) {
            toast.error('已上传文件的列名不一致，请检查后再继续')
            return
          }
        } catch (e) {
          toast.error('无法校验文件列名，请稍后重试')
          return
        } finally {
          setCheckingSchema(false)
        }
      }
      navigate({ to: '/datasets/create', search: { step: 3 as any } })
      return
    }
  }

  const handleCancel = () => navigate({ to: '/datasets' })
  const handlePrev = () => {
    if (currentStep === 2) {
      navigate({ to: '/datasets/create', search: { step: 1 as any } })
    } else if (currentStep === 3) {
      navigate({ to: '/datasets/create', search: { step: 2 as any } })
    }
  }

  // 第3步：索引设置状态
  type ColumnSetting = {
    id: string
    name: string
    type: string
    enableSearch: boolean
    enableAnswer: boolean
    note?: string
    // 原始后端字段名，用于刷新时做稳定匹配，保留本地改名
    sourceName?: string
  }
  const [columns, setColumns] = useState<ColumnSetting[]>([])
  const [multiKey, setMultiKey] = useState(true)
  const [similarity, setSimilarity] = useState<number[]>([0.2])
  const [maxRecall, setMaxRecall] = useState<number[]>([5])
  const [isSubmitting, setIsSubmitting] = useState(false)

  const updateColumn = (id: string, patch: Partial<ColumnSetting>) => {
    setColumns((prev) => prev.map((c) => (c.id === id ? { ...c, ...patch } : c)))
  }
  const finishImport = async () => {
    // 前置校验
    if (!name.trim()) {
      toast.warning('请填写知识库名称')
      return
    }
    if (!tableName.trim()) {
      toast.warning('请填写数据表名称')
      return
    }
    if (columns.length === 0) {
      toast.warning('请至少配置一个字段')
      return
    }
    // 使用用户填写的表名
    const normalizedTableName = tableName.trim()

    // 构造后端 schema
    const backendSchema: import('@/features/table-rag/data/schema').ColumnSchema[] = columns
      .map((c) => ({
        name: (c.name || '').trim(),
        type:
          c.type === 'date'
            ? 'datatime'
            : c.type === 'number'
              ? 'long'
              : (c.type as any),
        description: undefined,
        searchable: Boolean(c.enableSearch),
        retrievable: Boolean(c.enableAnswer),
      }))
      .filter((c) => !!c.name)

    if (backendSchema.length === 0) {
      toast.warning('字段名不可为空，请完善后重试')
      return
    }

    try {
      setIsSubmitting(true)
      const retrievalColumn = Array.from(
        new Set(
          columns
            .filter((c) => c.enableSearch)
            .map((c) => (c.name || '').trim())
            .filter((n) => n.length > 0)
        )
      ).join(',')
      const replyColumn = Array.from(
        new Set(
          columns
            .filter((c) => c.enableAnswer)
            .map((c) => (c.name || '').trim())
            .filter((n) => n.length > 0)
        )
      ).join(',')
      const payload: import('@/features/table-rag/data/schema').CreateDatasetRequest = {
        name: name.trim(),
        description: description.trim() || undefined,
        type: 'upload',
        table_name: normalizedTableName,
        schema: backendSchema,
        similarity_threshold: similarity[0],
        max_results: maxRecall[0],
        retrieval_column: retrievalColumn || undefined,
        reply_column: replyColumn || undefined,
      }

      // 创建知识库
      const created = await TableRagApiService.createDataset(payload)
      toast.success('知识库创建成功')

      // 为已上传文件创建入库任务
      const fileIds = uploaded.map((u) => u.id)
      for (const fid of fileIds) {
        try {
          const res = await TableRagApiService.ingestFile({ dataset_id: created.id, file_id: fid })
          if (res.task_id) {
            toast.message(`已创建入库任务：${res.task_id}`)
          }
        } catch (e: any) {
          // 单个文件失败不阻断整体流程，提示即可
          toast.error(e?.message || `创建入库任务失败（文件ID: ${fid}）`)
        }
      }

      // 完成后返回列表页
      navigate({ to: '/datasets' })
    } catch (e: any) {
      toast.error(e?.message || '创建知识库失败')
    } finally {
      setIsSubmitting(false)
    }
  }

  // 进入第3步时，通过后端接口预览表头并填充字段设置
  const mapType = (t: BackendColumnSchema['type']): ColumnSetting['type'] => {
    if (t === 'datatime') return 'date'
    if (t === 'long') return 'long'
    if (t === 'double') return 'double'
    return 'string'
  }
  
  // 防重复：开发环境下 React StrictMode 会双调用 effect；依赖变化也可能频繁触发
  const isFetchingSchemaRef = useRef(false)
  const lastSchemaKeyRef = useRef('')

  const fetchSchema = async (force = false) => {
    try {
      if (currentStep !== 3) return
      if (sourceType !== 'upload') return
      if (uploaded.length === 0) return
      const fileIds = uploaded.map(u => u.id)
      const schemaKey = `${sourceType}:${fileIds.join(',')}`
      if (schemaKey === lastSchemaKeyRef.current && !force) return
      if (isFetchingSchemaRef.current) return
      isFetchingSchemaRef.current = true
      lastSchemaKeyRef.current = schemaKey
      const schema = await TableRagApiService.previewSchema(fileIds)
      // 增量合并：保留本地编辑，按首次出现顺序追加新字段
      setColumns((prev) => {
        // 计算当前最大id以便为新增字段生成递增id
        const maxId = prev.reduce((m, c) => {
          const n = Number.parseInt(c.id, 10)
          return Number.isFinite(n) ? Math.max(m, n) : m
        }, 0)
        let nextId = maxId + 1

        // 后端字段按顺序索引
        const backendByName = new Map(schema.map((c) => [c.name, c]))

        // 先保留现有顺序并更新提示信息（note），不覆盖本地 name/type/开关
        const kept: ColumnSetting[] = prev.map((c) => {
          const key = c.sourceName || c.name
          const backend = backendByName.get(key)
          if (backend) {
            return {
              ...c,
              // 同步最新的冲突/说明提示
              note: backend.description,
              // 保持 sourceName 稳定
              sourceName: c.sourceName || backend.name,
            }
          }
          return c
        })

        // 已存在（按源字段名匹配）的集合，避免重复追加
        const existingKeys = new Set(
          kept.map((c) => (c.sourceName || c.name))
        )

        // 追加后端新增字段，按后端顺序
        const appended: ColumnSetting[] = []
        for (const c of schema) {
          if (existingKeys.has(c.name)) continue
          appended.push({
            id: String(nextId++),
            name: c.name,
            type: mapType(c.type),
            enableSearch: c.searchable ?? true,
            enableAnswer: c.retrievable ?? true,
            note: c.description,
            sourceName: c.name,
          })
        }

        const result = [...kept, ...appended]

        // 反馈合并结果
        const updatedCount = kept.filter((c) => {
          const key = c.sourceName || c.name
          return backendByName.has(key)
        }).length
        const addedCount = appended.length
        toast.message(`字段已刷新：新增 ${addedCount} 个，更新 ${updatedCount} 个（已保留本地编辑）`)

        if (schema.some(s => s.description)) {
          toast.info('存在类型冲突的字段已标注（仅更新提示不覆盖你的选择）')
        }

        return result
      })
    } catch (e: any) {
      console.error(e)
      toast.error(e?.message || '获取字段表头失败')
    } finally {
      isFetchingSchemaRef.current = false
    }
  }

  useEffect(() => {
    fetchSchema()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [currentStep, uploaded, sourceType])

  // 允许的文件类型与大小
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
  const removeFile = (idx: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== idx))
  }

  const removeUploaded = (id: string) => {
    setUploaded((prev) => prev.filter((u) => u.id !== id))
  }

  const clearUploaded = () => {
    setUploaded([])
  }

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
      // 清空选择（保留已上传列表）
      setFiles([])
    } catch (e: any) {
      toast.error(e?.message || '上传失败')
    } finally {
      setIsUploading(false)
    }
  }

  return (
    <>
      <Header fixed>
        <HeaderActions />
      </Header>
      <Main fixed fluid>
        {/* 顶部粘性工具栏：面包屑 + 居中步骤指示器 */}
        <div className='sticky top-0 z-40 border-b bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='flex items-center justify-between py-3'>
            <Breadcrumb>
              <BreadcrumbList>
                <BreadcrumbItem>
                  <BreadcrumbLink asChild>
                    <Link to='/datasets'>知识库</Link>
                  </BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                  <BreadcrumbPage>创建知识库</BreadcrumbPage>
                </BreadcrumbItem>
              </BreadcrumbList>
            </Breadcrumb>
          </div>
          {/* 居中固定步骤指示器 */}
          <div className='flex items-center justify-center gap-6 py-2 text-sm'>
            <div className={`flex items-center gap-2 ${currentStep === 1 ? '' : 'text-muted-foreground'}`}>
              <span className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-xs ${currentStep === 1 ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}>1</span>
              <span className={currentStep === 1 ? 'font-medium' : ''}>基础信息</span>
            </div>
            <div className={`flex items-center gap-2 ${currentStep === 2 ? '' : 'text-muted-foreground'}`}>
              <span className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-xs ${currentStep === 2 ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}>2</span>
              <span className={currentStep === 2 ? 'font-medium' : ''}>选择数据</span>
            </div>
            <div className={`flex items-center gap-2 ${currentStep === 3 ? '' : 'text-muted-foreground'}`}>
              <span className={`inline-flex h-6 w-6 items-center justify-center rounded-full text-xs ${currentStep === 3 ? 'bg-primary text-primary-foreground' : 'bg-muted'}`}>3</span>
              <span className={currentStep === 3 ? 'font-medium' : ''}>索引设置</span>
            </div>
          </div>
        </div>

        {/* 中间滚动区域 */}
        <div className='flex-1 overflow-auto'>

        {/* Step 1: 基础信息 */}
        {currentStep === 1 && (
          <Card className='border-none rounded-none'>
            <CardHeader>
              <CardTitle>基础信息</CardTitle>
            </CardHeader>
            <CardContent className='space-y-6 pb-24'>
            {/* 名称 */}
            <div className='grid gap-2'>
              <Label htmlFor='kb-name'>知识库名称<span className='text-destructive'> *</span></Label>
              <div className='flex items-center gap-2'>
                <Input
                  id='kb-name'
                  placeholder='请输入知识库名称'
                  value={name}
                  onChange={(e) => setName(e.target.value.slice(0, nameMax))}
                  maxLength={nameMax}
                />
                <span className='text-xs text-muted-foreground'>{name.length} / {nameMax}</span>
              </div>
            </div>

            {/* 描述 */}
            <div className='grid gap-2'>
              <Label htmlFor='kb-desc'>知识库描述</Label>
              <div className='flex items-start gap-2'>
                <Textarea
                  id='kb-desc'
                  placeholder='请输入知识库描述，介绍知识库包含的内容，用途等（可选）'
                  value={description}
                  onChange={(e) => setDescription(e.target.value.slice(0, descMax))}
                  maxLength={descMax}
                  className='min-h-24'
                />
                <span className='text-xs text-muted-foreground'>{description.length} / {descMax}</span>
              </div>
            </div>

            {/* 类型选择 */}
            <div className='grid gap-3'>
              <Label>知识库类型</Label>
              <RadioGroup value={kbType} onValueChange={(v) => setKbType(v as any)} className='grid grid-cols-1 md:grid-cols-3 gap-3'>
                <label className='flex cursor-not-allowed items-start gap-3 rounded-lg border p-3 opacity-60' htmlFor='kb-type-doc'>
                  <RadioGroupItem id='kb-type-doc' value='doc' disabled />
                  <div>
                    <div className='flex items-center gap-2'>
                      <FileText className='h-4 w-4 text-primary' />
                      <span className='font-medium'>文档搜索</span>
                    </div>
                    <div className='text-xs text-muted-foreground mt-1'>适配文件/切片构建索引的检索（暂未实现）。</div>
                  </div>
                </label>

                <label className='flex cursor-pointer items-start gap-3 rounded-lg border p-3 hover:bg-muted' htmlFor='kb-type-data'>
                  <RadioGroupItem id='kb-type-data' value='data' />
                  <div>
                    <div className='flex items-center gap-2'>
                      <Database className='h-4 w-4 text-primary' />
                      <span className='font-medium'>数据查询</span>
                    </div>
                    <div className='text-xs text-muted-foreground mt-1'>结构化数据检索与查询。</div>
                  </div>
                </label>

                <label className='flex cursor-not-allowed items-start gap-3 rounded-lg border p-3 opacity-60' htmlFor='kb-type-image'>
                  <RadioGroupItem id='kb-type-image' value='image' disabled />
                  <div>
                    <div className='flex items-center gap-2'>
                      <ImageIcon className='h-4 w-4 text-primary' />
                      <span className='font-medium'>图片问答</span>
                    </div>
                    <div className='text-xs text-muted-foreground mt-1'>基于图片Embedding的信息检索（暂未实现）。</div>
                  </div>
                </label>
              </RadioGroup>
            </div>

            </CardContent>
          </Card>
        )}

        {/* Step 2: 选择数据（shadcn 上传） */}
        {currentStep === 2 && (
          <Card className='border-none rounded-none'>
            <CardHeader>
              <CardTitle>选择数据</CardTitle>
            </CardHeader>
            <CardContent className='space-y-6 pb-24'>
              {/* 数据来源 */}
              <div className='grid gap-3'>
                <Label>数据来源<span className='text-destructive'> *</span></Label>
                <RadioGroup value={sourceType} onValueChange={(v) => setSourceType(v as any)} className='grid grid-cols-1 md:grid-cols-3 gap-3'>
                  <label className='flex cursor-pointer items-start gap-3 rounded-lg border p-3 hover:bg-muted' htmlFor='src-upload'>
                    <RadioGroupItem id='src-upload' value='upload' />
                    <div>
                      <div className='flex items-center gap-2'>
                        <UploadCloud className='h-4 w-4 text-primary' />
                        <span className='font-medium'>上传数据集</span>
                      </div>
                      <div className='text-xs text-muted-foreground mt-1'>
                        直接上传本地文件，支持多种格式。
                      </div>
                    </div>
                  </label>
                  <label className='flex cursor-pointer items-start gap-3 rounded-lg border p-3 hover:bg-muted' htmlFor='src-select'>
                    <RadioGroupItem id='src-select' value='select' />
                    <div>
                      <div className='flex items-center gap-2'>
                        <Database className='h-4 w-4 text-primary' />
                        <span className='font-medium'>选数据源表</span>
                      </div>
                      <div className='text-xs text-muted-foreground mt-1'>
                        从已接入的数据源选择表或视图。
                      </div>
                    </div>
                  </label>
                  <label className='flex cursor-pointer items-start gap-3 rounded-lg border p-3 hover:bg-muted' htmlFor='src-related'>
                    <RadioGroupItem id='src-related' value='related' />
                    <div>
                      <div className='flex items-center gap-2'>
                        <Database className='h-4 w-4 text-primary' />
                        <span className='font-medium'>关联数据源</span>
                      </div>
                      <div className='text-xs text-muted-foreground mt-1'>
                        关联RDS/外部数据源。
                      </div>
                    </div>
                  </label>
                </RadioGroup>
              </div>

              {/* 数据表名称 */}
              <div className='grid gap-2'>
                <Label>数据表名称<span className='text-destructive'> *</span></Label>
                <Input
                  placeholder='请输入数据表名称，例如: sales_2024'
                  value={tableName}
                  // 限制只能输入英文字符或下划线（自动过滤非法字符）
                  onChange={(e) => {
                    const raw = e.target.value
                    const sanitized = raw.replace(/[^A-Za-z_]/g, '')
                    setTableName(sanitized)
                  }}
                  pattern='^[A-Za-z_]+$'
                  title="仅支持英文字符或 '_'"
                />
                <div className='text-xs text-muted-foreground'>用于知识库标识，不同知识库建议不同表名（仅支持英文字符或 '_'）。</div>
              </div>

              {/* 上传区域：shadcn 组件 + 拖拽 */}
              {sourceType === 'upload' && (
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
                </div>
              )}

            </CardContent>
          </Card>
        )}

        {/* Step 3: 索引设置 */}
        {currentStep === 3 && (
          <Card className='border-none rounded-none'>
            <CardHeader>
              <CardTitle>索引设置</CardTitle>
            </CardHeader>
            <CardContent className='space-y-6 pb-24'>
              {/* 字段设置 */}
              <div className='space-y-3'>
                <div className='flex items-center justify-between'>
                  <div>
                    <Label className='text-base'>字段设置</Label>
                    <div className='text-xs text-muted-foreground'>配置参与检索/回答的字段</div>
                  </div>
                  <div className='flex items-center gap-2'>
                    <Button
                      variant='secondary'
                      size='sm'
                      onClick={() => fetchSchema(true)}
                      disabled={sourceType !== 'upload' || uploaded.length === 0}
                      title={sourceType !== 'upload' ? '仅支持上传数据源刷新字段' : (uploaded.length === 0 ? '请先在第2步上传至少一个文件' : '刷新字段')}
                    >
                      刷新字段
                    </Button>
                  </div>
                </div>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className='w-10'>序</TableHead>
                      <TableHead>字段名</TableHead>
                      <TableHead>字段类型</TableHead>
                      <TableHead className='text-center'>参与检索</TableHead>
                      <TableHead className='text-center'>参与回答</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {columns.map((c, idx) => (
                      <TableRow key={c.id}>
                        <TableCell className='text-muted-foreground'>{idx + 1}</TableCell>
                        <TableCell>
                          <div className='space-y-1'>
                            <Input value={c.name} onChange={(e) => updateColumn(c.id, { name: e.target.value })} />
                            {c.note && (
                              <div className='text-[11px] text-amber-600 dark:text-amber-500'>
                                {c.note}
                              </div>
                            )}
                          </div>
                        </TableCell>
                        <TableCell>
                          <Select value={c.type === 'number' ? 'long' : c.type} onValueChange={(v) => updateColumn(c.id, { type: v })}>
                            <SelectTrigger className='w-36'>
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value='string'>string</SelectItem>
                              <SelectItem value='long'>long</SelectItem>
                              <SelectItem value='double'>double</SelectItem>
                              <SelectItem value='date'>date</SelectItem>
                            </SelectContent>
                          </Select>
                        </TableCell>
                        <TableCell className='text-center'>
                          <Switch checked={c.enableSearch} onCheckedChange={(v) => updateColumn(c.id, { enableSearch: Boolean(v) })} />
                        </TableCell>
                        <TableCell className='text-center'>
                          <Switch checked={c.enableAnswer} onCheckedChange={(v) => updateColumn(c.id, { enableAnswer: Boolean(v) })} />
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              {/* 检索参数 */}
              <div className='grid grid-cols-1 gap-6'>
                <div className='space-y-3'>
                  <Label>多字段关键词匹配</Label>
                  <div className='flex items-center gap-3'>
                    <Switch checked={multiKey} onCheckedChange={(v) => setMultiKey(Boolean(v))} />
                    <span className='text-sm text-muted-foreground'>开启后关键词会在多个字段中匹配</span>
                  </div>
                </div>
                <div className='space-y-3'>
                  <Label>向量模型</Label>
                  <Alert>
                    <AlertDescription>
                      默认是 <code>text-embedding-v4</code>
                    </AlertDescription>
                  </Alert>
                </div>

                {/* 排序方式选项已移除 */}

                <div className='space-y-2'>
                  <div className='flex items-center justify-between'>
                    <Label>相似度阈值</Label>
                    <span className='text-xs text-muted-foreground'>{similarity[0].toFixed(2)}</span>
                  </div>
                  <Slider value={similarity} onValueChange={setSimilarity} min={0.01} max={1} step={0.01} />
                </div>

                <div className='space-y-2'>
                  <div className='flex items-center justify-between'>
                    <Label>最大召回数量</Label>
                    <span className='text-xs text-muted-foreground'>{maxRecall[0]}</span>
                  </div>
                  <Slider value={maxRecall} onValueChange={setMaxRecall} min={1} max={20} step={1} />
                </div>
              </div>

              {/* 向量存储选项已移除 */}

            </CardContent>
          </Card>
        )}
        </div>

        {/* 底部粘性操作栏 */}
        <div className='sticky bottom-0 z-40 border-t bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='flex items-center justify-between pt-2 pb-2'>
            <div>
              {currentStep > 1 && (
                <Button variant='outline' onClick={handlePrev}>上一步</Button>
              )}
            </div>
            <div className='flex items-center gap-3'>
              <Button variant='outline' onClick={handleCancel}>取消</Button>
              {currentStep === 1 && (
                <Button onClick={handleNext} disabled={!name.trim()}>下一步</Button>
              )}
              {currentStep === 2 && (
                <Button onClick={handleNext} disabled={(sourceType === 'upload' && uploaded.length === 0) || checkingSchema}>
                  {checkingSchema ? '检查中...' : '下一步'}
                </Button>
              )}
              {currentStep === 3 && (
                <Button onClick={finishImport} disabled={isSubmitting}>
                  {isSubmitting ? '提交中...' : '导入完成'}
                </Button>
              )}
            </div>
          </div>
        </div>
      </Main>
    </>
  )
}

export const Route = createFileRoute('/_authenticated/datasets/create')({
  validateSearch: (search) => {
    const raw = (search as any)?.step
    const s = Number(raw)
    return { step: Number.isFinite(s) && (s === 2 || s === 3) ? s : 1 }
  },
  component: CreateDatasetPage,
})