export interface SearchParams {
  query: string
  project_id: string
  max_results: number
  vector_weight: number
  keyword_weight: number
  enable_vector_search: boolean
  enable_keyword_search: boolean
  search_mode: 'vector' | 'keyword' | 'hybrid'
  // 过滤条件
  methods: string[]
  tags: string[]
  domain: string
  path_prefix: string
  filters?: Record<string, any>
}

// 后端返回的接口数据结构
export interface ApiInterface {
  path: string
  method: string
  summary: string
  description: string
  operation_id: string | null
  path_params: any[]
  query_params: any[]
  header_params: any[]
  body_params: any[]
  request_schema: any
  response_schema: string
  tags: string[]
  domain: string | null
  deprecated: boolean
  service_description: string
  embedding: number[]
  embedding_model: string
}

// 后端返回的搜索结果项
export interface SearchResultItem {
  project_id: string
  interface: ApiInterface
  score: number
  match_reason: string
  similarity_score: number | null
  search_type: string
}

// 后端返回的完整响应结构
export interface SearchResponse {
  interfaces: SearchResultItem[]
  query_time_ms: number
  total_count: number
  search_mode: string
}

// 前端使用的搜索结果格式（转换后的）
export interface SearchResult {
  score: number
  summary: string
  method: string
  path: string
  description: string
  service_description: string
  match_reason: string
  search_type: string
  [key: string]: any
}

export const defaultSearchParams: SearchParams = {
  query: '',
  project_id: 'agent-bot',
  max_results: 5,
  vector_weight: 0.5,
  keyword_weight: 0.5,
  enable_vector_search: true,
  enable_keyword_search: true,
  search_mode: 'hybrid',
  // 过滤条件默认值
  methods: [],
  tags: [],
  domain: '',
  path_prefix: '',
  filters: {}
}