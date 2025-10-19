// 搜索类型枚举，匹配后端SearchType
export type SearchType = 'Vector' | 'Keyword' | 'Hybrid'

// 项目信息接口
export interface ProjectInfo {
  id: string
  name: string
  description?: string
  status: string
}

// 过滤条件接口
export interface SearchFilters {
  project_id?: string
  methods?: string[]
  domain?: string
  path_prefix?: string
}

// 前端搜索参数接口，匹配后端InterfaceSearchRequest
export interface SearchParams {
  query: string
  search_type: SearchType
  max_results: number
  similarity_threshold?: number
  vector_weight?: number
  filters?: SearchFilters
}

// API参数定义
export interface ApiParameter {
  name: string
  param_type: string
  required: boolean
  description?: string
  example?: string
  default_value?: string
  enum_values?: string[]
  format?: string
}

// 后端返回的接口数据结构，匹配后端ApiInterface
export interface ApiInterface {
  path: string
  method: string
  summary?: string
  description?: string
  operation_id?: string
  path_params: ApiParameter[]
  query_params: ApiParameter[]
  header_params: ApiParameter[]
  body_params: ApiParameter[]
  request_schema?: string
  response_schema?: string
  tags: string[]
  domain?: string
  deprecated: boolean
  service_description?: string
  embedding?: number[]
  embedding_model?: string
  embedding_updated_at?: string
}

// 后端返回的搜索结果项，匹配后端InterfaceWithScore
export interface InterfaceWithScore {
  project_id?: string
  interface: ApiInterface
  score: number
  match_reason: string
}

// 后端返回的完整响应结构，匹配后端InterfaceSearchResponse
export interface InterfaceSearchResponse {
  interfaces: InterfaceWithScore[]
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
  search_type: 'Hybrid',
  max_results: 10,
  similarity_threshold: 0.7,
  vector_weight: 0.5,
  filters: {
    project_id: undefined,
    methods: [],
    domain: '',
    path_prefix: ''
  }
}