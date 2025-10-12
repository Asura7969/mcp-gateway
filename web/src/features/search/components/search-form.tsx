import { useState, useRef, useEffect } from 'react'
import { CaretSortIcon, CheckIcon } from '@radix-ui/react-icons'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
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
import { type SearchParams, defaultSearchParams } from '../data/schema'

// 项目列表数据
const projects = [
  { label: 'agent-bot', value: 'agent-bot' },
  { label: 'web-crawler', value: 'web-crawler' },
  { label: 'data-processor', value: 'data-processor' },
  { label: 'api-gateway', value: 'api-gateway' },
  { label: 'ml-pipeline', value: 'ml-pipeline' },
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
  const [tagInput, setTagInput] = useState('')
  const [popoverWidth, setPopoverWidth] = useState<number>(200)
  const [projectPopoverWidth, setProjectPopoverWidth] = useState<number>(200)
  const buttonRef = useRef<HTMLButtonElement>(null)
  const projectButtonRef = useRef<HTMLButtonElement>(null)

  useEffect(() => {
    if (buttonRef.current) {
      setPopoverWidth(buttonRef.current.offsetWidth)
    }
  }, [filtersOpen])

  useEffect(() => {
    if (projectButtonRef.current) {
      setProjectPopoverWidth(projectButtonRef.current.offsetWidth)
    }
  }, [params.project_id])

  const handleSliderChange = (value: number) => {
    const vectorWeight = value
    const keywordWeight = 1 - value
    
    let searchMode: 'vector' | 'keyword' | 'hybrid' = 'hybrid'
    let enableVector = true
    let enableKeyword = true
    
    if (value === 0) {
      searchMode = 'keyword'
      enableVector = false
      enableKeyword = true
    } else if (value === 1) {
      searchMode = 'vector'
      enableVector = true
      enableKeyword = false
    }
    
    setParams(prev => ({
      ...prev,
      vector_weight: vectorWeight,
      keyword_weight: keywordWeight,
      enable_vector_search: enableVector,
      enable_keyword_search: enableKeyword,
      search_mode: searchMode
    }))
  }

  // 处理methods过滤条件
  const handleMethodToggle = (method: string) => {
    setParams(prev => ({
      ...prev,
      methods: prev.methods.includes(method)
        ? prev.methods.filter(m => m !== method)
        : [...prev.methods, method]
    }))
  }



  // 移除tag
  const handleRemoveTag = (tag: string) => {
    setParams(prev => ({
      ...prev,
      tags: prev.tags.filter(t => t !== tag)
    }))
  }

  // 添加新标签
  const handleAddTag = () => {
    const trimmedTag = tagInput.trim()
    if (trimmedTag && !params.tags.includes(trimmedTag)) {
      setParams(prev => ({
        ...prev,
        tags: [...prev.tags, trimmedTag]
      }))
      setTagInput('')
    }
  }

  // 处理标签输入框的键盘事件
  const handleTagInputKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleAddTag()
    }
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
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

          {/* Project ID选择和Max Results输入 - 同一行 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="project_id">项目ID</Label>
              <Popover>
                <PopoverTrigger asChild>
                  <Button
                    ref={projectButtonRef}
                    variant="outline"
                    role="combobox"
                    className={cn(
                      "w-full justify-between",
                      !params.project_id && "text-muted-foreground"
                    )}
                  >
                    {params.project_id
                      ? projects.find((project) => project.value === params.project_id)?.label
                      : "选择项目"}
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
                        {projects.map((project) => (
                          <CommandItem
                            value={project.label}
                            key={project.value}
                            onSelect={() => {
                              setParams(prev => ({ ...prev, project_id: project.value }))
                            }}
                          >
                            <CheckIcon
                              className={cn(
                                "mr-2 h-4 w-4",
                                project.value === params.project_id
                                  ? "opacity-100"
                                  : "opacity-0"
                              )}
                            />
                            {project.label}
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
                max="10"
                value={params.max_results}
                onChange={(e) => setParams(prev => ({ 
                  ...prev, 
                  max_results: Math.min(10, Math.max(1, parseInt(e.target.value) || 5))
                }))}
              />
            </div>
          </div>

          {/* 搜索权重滑块 */}
          <div className="space-y-4">
            <Label>搜索权重</Label>
            <div className="space-y-2">
              <div className="flex justify-between text-sm text-muted-foreground">
                <span>向量搜索</span>
                <span>关键词搜索</span>
              </div>
              <div className="relative">
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  value={params.vector_weight}
                  onChange={(e) => handleSliderChange(parseFloat(e.target.value))}
                  className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
                />
              </div>
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>{params.vector_weight.toFixed(1)}</span>
                <span>{params.keyword_weight.toFixed(1)}</span>
              </div>

            </div>
          </div>

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
                    {params.methods.length > 0 && (
                      <div className="flex flex-wrap gap-1">
                        {params.methods.map((method) => (
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
                                      params.methods.includes(method) ? "opacity-100" : "opacity-0"
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

                  {/* Tags 过滤条件 */}
                  <div className="space-y-2">
                    <Label className="text-sm font-medium">标签</Label>
                    {/* 已选择的标签 */}
                    {params.tags.length > 0 && (
                      <div className="flex flex-wrap gap-1">
                        {params.tags.map((tag) => (
                          <Badge key={tag} variant="secondary" className="text-xs">
                            {tag}
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-auto p-0 ml-1"
                              onClick={() => handleRemoveTag(tag)}
                            >
                              <X className="h-3 w-3" />
                            </Button>
                          </Badge>
                        ))}
                      </div>
                    )}
                    {/* 标签输入框 */}
                    <div className="flex gap-2">
                      <Input
                        placeholder="输入标签名称，按回车添加"
                        value={tagInput}
                        onChange={(e) => setTagInput(e.target.value)}
                        onKeyDown={handleTagInputKeyDown}
                        className="flex-1"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={handleAddTag}
                        disabled={!tagInput.trim() || params.tags.includes(tagInput.trim())}
                      >
                        添加
                      </Button>
                    </div>
                  </div>

                  {/* Domain 和 Path Prefix 输入框 */}
                  <div className="grid grid-cols-2 gap-4">
                    <div className="space-y-2">
                      <Label htmlFor="domain" className="text-sm font-medium">域名</Label>
                      <Input
                        id="domain"
                        placeholder="例如: api.example.com"
                        value={params.domain}
                        onChange={(e) => setParams(prev => ({ ...prev, domain: e.target.value }))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="path_prefix" className="text-sm font-medium">路径前缀</Label>
                      <Input
                        id="path_prefix"
                        placeholder="例如: /api/v1"
                        value={params.path_prefix}
                        onChange={(e) => setParams(prev => ({ ...prev, path_prefix: e.target.value }))}
                      />
                    </div>
                  </div>
                </CardContent>
              </Card>
            </CollapsibleContent>
            </Collapsible>

          {/* 搜索按钮 */}
          <Button type="submit" className="w-full" disabled={loading || !params.query.trim()}>
            <Search className="w-4 h-4 mr-2" />
            {loading ? '搜索中...' : '搜索'}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}