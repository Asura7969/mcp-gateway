import { useState } from 'react'
import { Header } from '@/components/layout/header'
import { Main } from '@/components/layout/main'
import { ProfileDropdown } from '@/components/profile-dropdown'
import { Search as SearchComponent } from '@/components/search'
import { ThemeSwitch } from '@/components/theme-switch'
import { ConfigDrawer } from '@/components/config-drawer'
import { SearchForm } from './components/search-form'
import { SearchResults } from './components/search-results'
import { type SearchParams, type SearchResult } from './data/schema'
import { SearchApiService } from './data/api'

export function SearchDebug() {
  const [searchResults, setSearchResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [queryTime, setQueryTime] = useState<number | undefined>()
  const [totalCount, setTotalCount] = useState<number | undefined>()
  const [searchMode, setSearchMode] = useState<string | undefined>()

  const handleSearch = async (params: SearchParams) => {
    setLoading(true)
    setError(null)
    try {
      const response = await SearchApiService.search(params)
      setSearchResults(response.results)
      setQueryTime(response.queryTime)
      setTotalCount(response.totalCount)
      setSearchMode(response.searchMode)
    } catch (err) {
      setError(err instanceof Error ? err.message : '搜索失败')
      setSearchResults([])
      setQueryTime(undefined)
      setTotalCount(undefined)
      setSearchMode(undefined)
    } finally {
      setLoading(false)
    }
  }

  return (
    <>
      <Header>
        <SearchComponent />
        <div className='ms-auto flex items-center space-x-4'>
          <ThemeSwitch />
          <ConfigDrawer />
          <ProfileDropdown />
        </div>
      </Header>
      <Main fluid>
        <div className='mb-2 flex items-center justify-between space-y-2'>
          <div>
            <h2 className='text-2xl font-bold tracking-tight'>搜索调试</h2>
            <p className='text-muted-foreground'>
              调试接口检索效果，输入关键信息查询匹配的接口
            </p>
          </div>
        </div>
        
        <div className="flex gap-6 min-h-[600px]">
          <div className="w-2/5 min-w-[400px]">
            <SearchForm onSearch={handleSearch} loading={loading} />
          </div>
          
          <div className="flex-1">
            <SearchResults 
              results={searchResults} 
              loading={loading}
              queryTime={queryTime}
              totalCount={totalCount}
              searchMode={searchMode}
            />
          </div>
        </div>
      </Main>
    </>
  )
}