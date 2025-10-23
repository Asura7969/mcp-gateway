import { useState, useEffect, useCallback, useRef } from 'react'
import { Header } from '@/components/layout/header'
import { Main } from '@/components/layout/main'
import { ProfileDropdown } from '@/components/profile-dropdown'
import { Search } from '@/components/search'
import { ThemeSwitch } from '@/components/theme-switch'
import { ConfigDrawer } from '@/components/config-drawer'
import { Button } from '@/components/ui/button'
import { EndpointsTable } from './components/endpoints-table'
import { CreateEndpointDialog } from './components/create-endpoint-dialog'
import { type Endpoint } from './data/schema'
import { EndpointsApiService } from './data/api'

export function Endpoints() {
  const [data, setData] = useState<Endpoint[]>([])
  const [loading, setLoading] = useState(true)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
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

      <Main fluid>
        <div className='mb-2 flex flex-wrap items-center justify-between space-y-2 gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>Endpoints</h2>
            <p className='text-muted-foreground'>
              管理您的MCP端点服务
            </p>
          </div>
          <Button onClick={() => setIsCreateDialogOpen(true)}>
            Create
          </Button>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 lg:flex-row lg:space-y-0 lg:space-x-12'>
          <EndpointsTable
            data={data}
            onDataReload={() => fetchData(searchQuery)}
            onSearch={handleSearch}
            searchQuery={searchQuery}
            loading={loading}
          />
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