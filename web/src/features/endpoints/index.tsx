import { useState, useEffect } from 'react'
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

  useEffect(() => {
    const fetchData = async () => {
      try {
        // 使用分页 API 获取端点数据
        const response = await EndpointsApiService.getEndpointsPaginated()
        setData(response.endpoints)
      } catch (error) {
        console.error('Failed to fetch endpoints:', error)
        // 如果 API 调用失败，使用 mock 数据
        // setData(endpoints)
      } finally {
        setLoading(false)
      }
    }

    fetchData()
  }, [])

  if (loading) {
    return <div>Loading...</div>
  }

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

      <Main>
        <div className='mb-2 flex flex-wrap items-center justify-between space-y-2 gap-x-4'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>端点管理</h2>
            <p className='text-muted-foreground'>
              管理您的MCP端点服务
            </p>
          </div>
          <Button onClick={() => setIsCreateDialogOpen(true)}>
            创建端点
          </Button>
        </div>
        <div className='-mx-4 flex-1 overflow-auto px-4 py-1 lg:flex-row lg:space-y-0 lg:space-x-12'>
          <EndpointsTable data={data} />
        </div>
      </Main>

      <CreateEndpointDialog 
        open={isCreateDialogOpen} 
        onOpenChange={setIsCreateDialogOpen} 
      />
    </>
  )
}