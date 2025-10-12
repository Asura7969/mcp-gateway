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
import { type SearchResult } from '../data/schema'

interface SearchResultsProps {
  results: SearchResult[]
  loading: boolean
}

export function SearchResults({ results, loading }: SearchResultsProps) {
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
          <p className="text-sm text-muted-foreground">
            找到 {results.length} 个匹配结果
          </p>
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
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-[80px]">评分</TableHead>
                  <TableHead className="w-[200px]">方法标题</TableHead>
                  <TableHead className="w-[100px]">方法</TableHead>
                  <TableHead className="w-[200px]">路径</TableHead>
                  <TableHead>描述</TableHead>
                  <TableHead>服务描述</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {results.map((result, index) => (
                  <TableRow key={index}>
                    <TableCell>
                      <Badge 
                        variant={result.score > 0.8 ? 'default' : result.score > 0.6 ? 'secondary' : 'outline'}
                        className="font-mono"
                      >
                        {result.score.toFixed(3)}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-medium">
                      <div className="max-w-[200px] truncate" title={result.summary}>
                        {result.summary}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="font-mono">
                        {result.method}
                      </Badge>
                    </TableCell>
                    <TableCell className="font-mono text-sm">
                      <div className="max-w-[200px] truncate" title={result.path}>
                        {result.path}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="max-w-[300px] truncate" title={result.description}>
                        {result.description}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div className="max-w-[300px] truncate" title={result.service_description}>
                        {result.service_description}
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}
      </CardContent>
    </Card>
  )
}