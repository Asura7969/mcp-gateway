import { useState, useEffect, useCallback, useRef } from 'react'
import { useReactTable, getCoreRowModel, getPaginationRowModel } from '@tanstack/react-table'
import { Header } from '@/components/layout/header'
import { Main } from '@/components/layout/main'
import { ProfileDropdown } from '@/components/profile-dropdown'
import { Search } from '@/components/search'
import { ThemeSwitch } from '@/components/theme-switch'
import { ConfigDrawer } from '@/components/config-drawer'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { DataTablePagination } from '@/components/data-table/pagination'
import { EndpointsTable } from './components/endpoints-table'
import { CreateEndpointDialog } from './components/create-endpoint-dialog'
import { type Endpoint } from './data/schema'
import { EndpointsApiService } from './data/api'

export function Endpoints() {
  const [data, setData] = useState<Endpoint[]>([])
  const [loading, setLoading] = useState(true)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [pagination, setPagination] = useState({
    pageIndex: 0,
    pageSize: 10,
  })
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null)

  const fetchData = async (search?: string) => {
    try {
      setLoading(true)
      // 使用分页 API 获取端点数据，传递搜索参数
      const response = await EndpointsApiService.getEndpointsPaginated(
        undefined, // page
        undefined, // pageSize
        search,    // search
        undefined  // status
      )
      setData(response.endpoints)
    } catch (error) {
      console.error('Failed to fetch endpoints:', error)
      // 如果 API 调用失败，使用 mock 数据
      // setData(endpoints)
    } finally {
      setLoading(false)
    }
  }

  const debouncedFetchData = useCallback((query: string) => {
    // 清除之前的定时器
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current)
    }

    // 设置新的定时器，500ms 后执行搜索
    debounceTimerRef.current = setTimeout(() => {
      fetchData(query)
    }, 500)
  }, [])

  const handleSearch = (query: string) => {
    setSearchQuery(query)
    // 如果查询为空，立即执行搜索
    if (query.trim() === '') {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current)
      }
      fetchData('')
    } else {
      // 否则使用防抖搜索
      debouncedFetchData(query)
    }
  }

  // 组件卸载时清理定时器
  useEffect(() => {
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current)
      }
    }
  }, [])

  useEffect(() => {
    fetchData()
  }, [])

  // 创建表格实例 - 简化版本，主要用于分页控制
  const table = useReactTable({
    data,
    columns: [
      { accessorKey: 'name', header: 'service' },
      { accessorKey: 'description', header: 'description' },
      { accessorKey: 'created_at', header: 'create time' },
      { id: 'actions', header: 'action' }
    ],
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    state: {
      pagination,
    },
    onPaginationChange: setPagination,
    manualPagination: false,
  })

  return (
    <>
      <Header>
        <Search />
        <div className='ms-auto flex items-center space-x-4'>
          <ThemeSwitch />
          <ConfigDrawer />
          <ProfileDropdown />
        </div>
      </Header>

      <Main fixed fluid>
        {/* 固定头部区域 */}
        <div className='sticky top-0 z-40 bg-background/80 backdrop-blur supports-[-webkit-backdrop-filter]:bg-background/60 supports-[backdrop-filter]:bg-background/60'>
          <div className='flex items-center justify-between py-3'>
            <div>
              <h2 className='text-2xl font-bold tracking-tight'>Endpoints</h2>
              <p className='text-muted-foreground'>
                管理您的MCP端点服务
              </p>
            </div>
            <div className='flex items-center gap-3'>
              <Input
                placeholder='search by service name...'
                value={searchQuery}
                onChange={(event) => handleSearch(event.target.value)}
                className='w-64'
              />
              <Button onClick={() => setIsCreateDialogOpen(true)}>
                Create
              </Button>
            </div>
          </div>
        </div>

        {/* 内容区域 - 可滚动 */}
        <div className='px-1 flex flex-col' style={{ height: 'calc(100vh - 240px)' }}>
          <div className='flex-1 overflow-auto'>
            <EndpointsTable
              data={data}
              onDataReload={() => fetchData(searchQuery)}
              loading={loading}
              table={table}
            />
          </div>
        </div>

        {/* 分页控制 - 固定在页面底部 */}
        <div className='bg-background p-3 flex-shrink-0'>
          <div className='max-w-7xl mx-auto'>
            <DataTablePagination table={table} />
          </div>
        </div>
      </Main>

      <CreateEndpointDialog
        open={isCreateDialogOpen}
        onOpenChange={setIsCreateDialogOpen}
        onSuccess={fetchData}
      />
    </>
  )
}