import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Skeleton } from '@/components/ui/skeleton'
import { Button } from '@/components/ui/button'
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible'
import { ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
import { cn } from '@/lib/utils'
import { type SearchResult } from '../data/schema'

interface SearchResultsProps {
  results: SearchResult[]
  loading: boolean
  queryTime?: number
  totalCount?: number
  searchMode?: string
}

export function SearchResults({ results, loading, queryTime, totalCount, searchMode }: SearchResultsProps) {
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set())

  const toggleRowExpansion = (index: number) => {
    const newExpanded = new Set(expandedRows)
    if (newExpanded.has(index)) {
      newExpanded.delete(index)
    } else {
      newExpanded.add(index)
    }
    setExpandedRows(newExpanded)
  }

  if (loading) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle>搜索结果</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="space-y-2">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-3/4" />
                <Skeleton className="h-4 w-1/2" />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle>搜索结果</CardTitle>
        {results.length > 0 && (
          <div className="flex flex-wrap gap-4 text-sm text-muted-foreground">
            <span>找到 {results.length} 个匹配结果</span>
            {totalCount && totalCount !== results.length && (
              <span>（共 {totalCount} 个）</span>
            )}
            {queryTime && (
              <span>查询耗时: {queryTime}ms</span>
            )}
            {searchMode && (
              <Badge variant="outline" className="text-xs">
                {searchMode}
              </Badge>
            )}
          </div>
        )}
      </CardHeader>
      <CardContent>
        {results.length === 0 ? (
          <div className="flex items-center justify-center h-64 text-muted-foreground">
            <div className="text-center">
              <p className="text-lg font-medium">暂无搜索结果</p>
              <p className="text-sm">请输入查询条件并点击搜索</p>
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            {results.map((result, index) => (
              <Card key={index} className="border">
                <CardContent className="p-4">
                  <div className="flex items-start justify-between">
                    <div className="flex-1 space-y-2">
                      {/* 基本信息行 */}
                      <div className="flex items-center gap-3">
                        <Badge 
                          variant={result.score > 0.8 ? 'default' : result.score > 0.6 ? 'secondary' : 'outline'}
                          className="font-mono text-xs"
                        >
                          {result.score.toFixed(3)}
                        </Badge>
                        <Badge variant="outline" className="font-mono text-xs">
                          {result.method}
                        </Badge>
                        <code className="text-sm bg-muted px-2 py-1 rounded">
                          {result.path}
                        </code>
                        {result.tags && result.tags.length > 0 && (
                           <div className="flex gap-1">
                             {result.tags.slice(0, 3).map((tag: string, tagIndex: number) => (
                               <Badge key={tagIndex} variant="secondary" className="text-xs">
                                 {tag}
                               </Badge>
                             ))}
                            {result.tags.length > 3 && (
                              <Badge variant="secondary" className="text-xs">
                                +{result.tags.length - 3}
                              </Badge>
                            )}
                          </div>
                        )}
                      </div>

                      {/* 标题和描述 */}
                      <div>
                        <h4 className="font-medium text-sm">{result.summary}</h4>
                        {result.description && (
                          <p className="text-sm text-muted-foreground mt-1">
                            {result.description}
                          </p>
                        )}
                      </div>

                      {/* 匹配原因 */}
                      {result.match_reason && (
                        <div className="text-xs text-muted-foreground bg-muted/50 px-2 py-1 rounded">
                          匹配原因: {result.match_reason}
                        </div>
                      )}
                    </div>

                    {/* 展开/收起按钮 */}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => toggleRowExpansion(index)}
                      className="ml-2"
                    >
                      {expandedRows.has(index) ? (
                        <ChevronDown className="h-4 w-4" />
                      ) : (
                        <ChevronRight className="h-4 w-4" />
                      )}
                    </Button>
                  </div>

                  {/* 展开的详细信息 */}
                  <Collapsible open={expandedRows.has(index)}>
                    <CollapsibleContent>
                      <div className="mt-4 pt-4 border-t space-y-4">
                        {/* 参数信息 */}
                        {(result.path_params?.length || result.query_params?.length || result.header_params?.length) && (
                          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                            {result.path_params && result.path_params.length > 0 && (
                              <div>
                                <h5 className="text-xs font-medium text-muted-foreground mb-2">路径参数</h5>
                                <div className="space-y-1">
                                   {result.path_params.map((param: any, paramIndex: number) => (
                                     <div key={paramIndex} className="text-xs">
                                       <code className="bg-muted px-1 rounded">{param.name}</code>
                                       {param.required && <span className="text-red-500 ml-1">*</span>}
                                       {param.description && (
                                         <span className="text-muted-foreground ml-2">{param.description}</span>
                                       )}
                                     </div>
                                   ))}
                                 </div>
                              </div>
                            )}

                            {result.query_params && result.query_params.length > 0 && (
                               <div>
                                 <h5 className="text-xs font-medium text-muted-foreground mb-2">查询参数</h5>
                                 <div className="space-y-1">
                                   {result.query_params.map((param: any, paramIndex: number) => (
                                     <div key={paramIndex} className="text-xs">
                                       <code className="bg-muted px-1 rounded">{param.name}</code>
                                       {param.required && <span className="text-red-500 ml-1">*</span>}
                                       {param.description && (
                                         <span className="text-muted-foreground ml-2">{param.description}</span>
                                       )}
                                     </div>
                                   ))}
                                 </div>
                               </div>
                             )}

                             {result.header_params && result.header_params.length > 0 && (
                               <div>
                                 <h5 className="text-xs font-medium text-muted-foreground mb-2">请求头参数</h5>
                                 <div className="space-y-1">
                                   {result.header_params.map((param: any, paramIndex: number) => (
                                     <div key={paramIndex} className="text-xs">
                                       <code className="bg-muted px-1 rounded">{param.name}</code>
                                       {param.required && <span className="text-red-500 ml-1">*</span>}
                                       {param.description && (
                                         <span className="text-muted-foreground ml-2">{param.description}</span>
                                       )}
                                     </div>
                                   ))}
                                 </div>
                               </div>
                             )}
                          </div>
                        )}

                        {/* Schema信息 */}
                        {(result.request_schema || result.response_schema) && (
                          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            {result.request_schema && (
                              <div>
                                <h5 className="text-xs font-medium text-muted-foreground mb-2">请求Schema</h5>
                                <pre className="text-xs bg-muted p-2 rounded overflow-x-auto">
                                  {JSON.stringify(result.request_schema, null, 2)}
                                </pre>
                              </div>
                            )}

                            {result.response_schema && (
                              <div>
                                <h5 className="text-xs font-medium text-muted-foreground mb-2">响应Schema</h5>
                                <pre className="text-xs bg-muted p-2 rounded overflow-x-auto">
                                  {JSON.stringify(result.response_schema, null, 2)}
                                </pre>
                              </div>
                            )}
                          </div>
                        )}

                        {/* 其他信息 */}
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs text-muted-foreground">
                          {result.service_description && (
                            <div>
                              <span className="font-medium">服务描述:</span>
                              <p className="mt-1">{result.service_description}</p>
                            </div>
                          )}
                          {result.embedding_model && (
                            <div>
                              <span className="font-medium">嵌入模型:</span>
                              <p className="mt-1">{result.embedding_model}</p>
                            </div>
                          )}
                          {result.embedding_updated_at && (
                            <div>
                              <span className="font-medium">嵌入更新时间:</span>
                              <p className="mt-1">{new Date(result.embedding_updated_at).toLocaleString()}</p>
                            </div>
                          )}
                          {result.search_type && (
                            <div>
                              <span className="font-medium">搜索类型:</span>
                              <p className="mt-1">{result.search_type}</p>
                            </div>
                          )}
                        </div>
                      </div>
                    </CollapsibleContent>
                  </Collapsible>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  )
}