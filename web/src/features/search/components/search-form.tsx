import { useState, useRef, useEffect } from 'react'
import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { ChevronDown, Search, X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { type SearchParams, type SearchType, type ProjectInfo, defaultSearchParams } from '../data/schema'
import { SearchApiService } from '../data/api'

// 搜索类型选项
const searchTypeOptions = [
  { label: '混合搜索', value: 'Hybrid' as SearchType },
  { label: '向量搜索', value: 'Vector' as SearchType },
  { label: '关键词搜索', value: 'Keyword' as SearchType },
] as const

const availableMethods = [
  'GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'
] as const

interface SearchFormProps {
  onSearch: (params: SearchParams) => void
  loading: boolean
}

export function SearchForm({ onSearch, loading }: SearchFormProps) {
  const [params, setParams] = useState<SearchParams>(defaultSearchParams)
  const [filtersOpen, setFiltersOpen] = useState(false)
  const [searchTypeOpen, setSearchTypeOpen] = useState(false)
  const [projectOpen, setProjectOpen] = useState(false)

  const [popoverWidth, setPopoverWidth] = useState<number>(200)
  const [searchTypePopoverWidth, setSearchTypePopoverWidth] = useState<number>(200)
  const [projectPopoverWidth, setProjectPopoverWidth] = useState<number>(200)
  const [projects, setProjects] = useState<ProjectInfo[]>([])
  const [projectsLoading, setProjectsLoading] = useState(false)
  const buttonRef = useRef<HTMLButtonElement>(null)
  const searchTypeButtonRef = useRef<HTMLButtonElement>(null)
  const projectButtonRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (buttonRef.current) {
      setPopoverWidth(buttonRef.current.offsetWidth)
    }
  }, [filtersOpen])

  useEffect(() => {
    if (searchTypeButtonRef.current) {
      setSearchTypePopoverWidth(searchTypeButtonRef.current.offsetWidth)
    }
  }, [params.search_type])

  useEffect(() => {
    if (projectButtonRef.current) {
      setProjectPopoverWidth(projectButtonRef.current.offsetWidth)
    }
  }, [params.filters?.project_id])

  // 获取项目列表
  useEffect(() => {
    const fetchProjects = async () => {
      setProjectsLoading(true)
      try {
        const projectList = await SearchApiService.getProjects()
        setProjects(projectList)
      } catch (error) {
        console.error('Failed to fetch projects:', error)
      } finally {
        setProjectsLoading(false)
      }
    }
    
    fetchProjects()
  }, [])

  // 处理methods过滤条件
  const handleMethodToggle = (method: string) => {
    setParams(prev => ({
      ...prev,
      filters: {
        ...prev.filters,
        methods: prev.filters?.methods?.includes(method)
          ? prev.filters.methods.filter(m => m !== method)
          : [...(prev.filters?.methods || []), method]
      }
    }))
  }



  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    
    // 验证项目选择是否为必填
    if (!params.filters?.project_id) {
      alert('请选择一个项目')
      return
    }
    
    onSearch(params)
  }

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>搜索条件</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Query输入 */}
          <div className="space-y-2">
            <Label htmlFor="query">查询内容</Label>
            <Textarea
              id="query"
              placeholder="输入查询关键词..."
              value={params.query}
              onChange={(e) => setParams(prev => ({ ...prev, query: e.target.value }))}
              className="min-h-[100px]"
            />
          </div>

          {/* 项目选择 */}
          <div className="space-y-2">
            <Label htmlFor="project_id">项目选择</Label>
            <Popover open={projectOpen} onOpenChange={setProjectOpen}>
              <PopoverTrigger asChild>
                <Button
                  ref={projectButtonRef}
                  variant="outline"
                  role="combobox"
                  className={cn(
                    "w-full justify-between",
                    !params.filters?.project_id && "text-muted-foreground"
                  )}
                  disabled={projectsLoading}
                >
                  {params.filters?.project_id
                    ? projects.find((project) => project.id === params.filters?.project_id)?.name || params.filters?.project_id
                    : projectsLoading ? "加载中..." : "选择项目（必填）"}
                  <CaretSortIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                </Button>
              </PopoverTrigger>
              <PopoverContent 
                className="p-0" 
                align="start"
                style={{ width: projectPopoverWidth }}
              >
                <Command>
                  <CommandInput placeholder="搜索项目..." />
                  <CommandEmpty>未找到项目。</CommandEmpty>
                  <CommandGroup>
                    <CommandList>
                      <CommandItem
                        value="all-projects"
                        onSelect={() => {
                          setParams(prev => ({ 
                            ...prev, 
                            filters: { ...prev.filters, project_id: undefined }
                          }))
                          setProjectOpen(false)
                        }}
                      >
                        <CheckIcon
                          className={cn(
                            "mr-2 h-4 w-4",
                            !params.filters?.project_id ? "opacity-100" : "opacity-0"
                          )}
                        />
                        所有项目
                      </CommandItem>
                      {projects.map((project) => (
                        <CommandItem
                          value={project.name}
                          key={project.id}
                          onSelect={() => {
                            setParams(prev => ({ 
                              ...prev, 
                              filters: { ...prev.filters, project_id: project.id }
                            }))
                            setProjectOpen(false)
                          }}
                        >
                          <CheckIcon
                            className={cn(
                              "mr-2 h-4 w-4",
                              project.id === params.filters?.project_id
                                ? "opacity-100"
                                : "opacity-0"
                            )}
                          />
                          {project.name}
                        </CommandItem>
                      ))}
                    </CommandList>
                  </CommandGroup>
                </Command>
              </PopoverContent>
            </Popover>
          </div>

          {/* 搜索类型选择和最大结果数 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="search_type">搜索类型</Label>
              <Popover open={searchTypeOpen} onOpenChange={setSearchTypeOpen}>
                <PopoverTrigger asChild>
                  <Button
                    ref={searchTypeButtonRef}
                    variant="outline"
                    role="combobox"
                    className={cn(
                      "w-full justify-between",
                      !params.search_type && "text-muted-foreground"
                    )}
                  >
                    {params.search_type
                      ? searchTypeOptions.find((option) => option.value === params.search_type)?.label
                      : "选择搜索类型"}
                    <CaretSortIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent 
                  className="p-0" 
                  align="start"
                  style={{ width: searchTypePopoverWidth }}
                >
                  <Command>
                    <CommandInput placeholder="搜索类型..." />
                    <CommandEmpty>未找到搜索类型。</CommandEmpty>
                    <CommandGroup>
                      <CommandList>
                        {searchTypeOptions.map((option) => (
                          <CommandItem
                            value={option.label}
                            key={option.value}
                            onSelect={() => {
                              setParams(prev => ({ ...prev, search_type: option.value }))
                              setSearchTypeOpen(false)
                            }}
                          >
                            <CheckIcon
                              className={cn(
                                "mr-2 h-4 w-4",
                                option.value === params.search_type
                                  ? "opacity-100"
                                  : "opacity-0"
                              )}
                            />
                            {option.label}
                          </CommandItem>
                        ))}
                      </CommandList>
                    </CommandGroup>
                  </Command>
                </PopoverContent>
              </Popover>
            </div>

            <div className="space-y-2">
              <Label htmlFor="max_results">最大返回数量</Label>
              <Input
                id="max_results"
                type="number"
                min="1"
                max="50"
                value={params.max_results}
                onChange={(e) => setParams(prev => ({ 
                  ...prev, 
                  max_results: Math.min(50, Math.max(1, parseInt(e.target.value) || 10))
                }))}
              />
            </div>
          </div>

          {/* 高级设置 */}
          {params.search_type === 'Hybrid' && (
            <div className="space-y-4">
              <Label>向量搜索权重</Label>
              <div className="space-y-2">
                <div className="flex justify-between text-sm text-muted-foreground">
                  <span>关键词搜索</span>
                  <span>向量搜索</span>
                </div>
                <div className="relative">
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={params.vector_weight || 0.5}
                    onChange={(e) => setParams(prev => ({ 
                      ...prev, 
                      vector_weight: parseFloat(e.target.value) 
                    }))}
                    className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
                  />
                </div>
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>{(1 - (params.vector_weight || 0.5)).toFixed(1)}</span>
                  <span>{(params.vector_weight || 0.5).toFixed(1)}</span>
                </div>
              </div>
            </div>
          )}

          {/* 相似度阈值 */}
          {(params.search_type === 'Vector' || params.search_type === 'Hybrid') && (
            <div className="space-y-2">
              <div className="flex justify-between items-center">
                <Label htmlFor="similarity_threshold">相似度阈值</Label>
                <span className="text-sm text-muted-foreground">
                  {(params.similarity_threshold || 0.7).toFixed(1)}
                </span>
              </div>
              <Slider
                id="similarity_threshold"
                min={0}
                max={1}
                step={0.1}
                value={[params.similarity_threshold || 0.7]}
                onValueChange={(value) => setParams(prev => ({ 
                  ...prev, 
                  similarity_threshold: value[0]
                }))}
                className="w-full"
              />
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>0.0</span>
                <span>1.0</span>
              </div>
            </div>
          )}

          {/* 过滤条件折叠面板 */}
          <Collapsible open={filtersOpen} onOpenChange={setFiltersOpen}>
            <CollapsibleTrigger asChild>
              <Button
                variant="ghost"
                className="w-full justify-between"
                type="button"
              >
                过滤条件
                <ChevronDown 
                  className={cn(
                    "h-4 w-4 transition-transform duration-200",
                    filtersOpen ? "rotate-180" : ""
                  )}
                />
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <Card className="rounded-none">
                <CardContent className="space-y-4">
                  {/* Methods 过滤条件 */}
                  <div className="space-y-2">
                    <Label className="text-sm font-medium">请求方法</Label>
                    {/* 已选择的方法 */}
                    {(params.filters?.methods?.length || 0) > 0 && (
                      <div className="flex flex-wrap gap-1">
                        {params.filters?.methods?.map((method) => (
                          <Badge key={method} variant="secondary" className="text-xs">
                            {method}
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-auto p-0 ml-1"
                              onClick={() => handleMethodToggle(method)}
                            >
                              <X className="h-3 w-3" />
                            </Button>
                          </Badge>
                        ))}
                      </div>
                    )}
                    {/* Methods Combobox */}
                    <Popover>
                      <PopoverTrigger asChild>
                        <Button
                          ref={buttonRef}
                          variant="outline"
                          role="combobox"
                          className="w-full justify-between"
                        >
                          选择请求方法
                          <CaretSortIcon className="ml-2 h-4 w-4 shrink-0 opacity-50" />
                        </Button>
                      </PopoverTrigger>
                      <PopoverContent 
                        className="p-0" 
                        align="start"
                        style={{ width: popoverWidth }}
                      >
                        <Command>
                          <CommandInput placeholder="搜索请求方法..." />
                          <CommandEmpty>未找到请求方法</CommandEmpty>
                          <CommandGroup>
                            <CommandList>
                              {availableMethods.map((method) => (
                                <CommandItem
                                  key={method}
                                  value={method}
                                  onSelect={() => handleMethodToggle(method)}
                                >
                                  <CheckIcon
                                    className={cn(
                                      "mr-2 h-4 w-4",
                                      params.filters?.methods?.includes(method) ? "opacity-100" : "opacity-0"
                                    )}
                                  />
                                  {method}
                                </CommandItem>
                              ))}
                            </CommandList>
                          </CommandGroup>
                        </Command>
                      </PopoverContent>
                    </Popover>
                  </div>



                  {/* Domain 和 Path Prefix 输入框 */}
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label htmlFor="domain" className="text-sm font-medium">域名</Label>
                      <Input
                        id="domain"
                        placeholder="例如: api.example.com"
                        value={params.filters?.domain || ''}
                        onChange={(e) => setParams(prev => ({ 
                          ...prev, 
                          filters: { ...prev.filters, domain: e.target.value }
                        }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="path_prefix" className="text-sm font-medium">路径前缀</Label>
                      <Input
                        id="path_prefix"
                        placeholder="例如: /api/v1"
                        value={params.filters?.path_prefix || ''}
                        onChange={(e) => setParams(prev => ({ 
                          ...prev, 
                          filters: { ...prev.filters, path_prefix: e.target.value }
                        }))}
                      />
                    </div>
                  </div>


                </CardContent>
              </Card>
            </CollapsibleContent>
          </Collapsible>

          {/* 搜索按钮 */}
          <Button 
            type="submit" 
            className="w-full" 
            disabled={loading || !params.query.trim() || !params.filters?.project_id}
          >
            <Search className="w-4 h-4 mr-2" />
            {loading ? '搜索中...' : '搜索'}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}