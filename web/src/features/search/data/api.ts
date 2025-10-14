import axios from 'axios'
import { type SearchParams, type SearchResult, type SearchResponse } from './schema'

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
  timeout: 10000,
})

export class SearchApiService {
  static async search(params: SearchParams): Promise<SearchResult[]> {
    const response = await api.post('/api/interface-retrieval/search', params)
    const data: SearchResponse = response.data
    
    // 转换后端响应数据为前端期望的格式
    return data.interfaces.map(item => ({
      score: item.score,
      summary: item.interface.summary || '无标题',
      method: item.interface.method,
      path: item.interface.path,
      description: item.interface.description || '无描述',
      service_description: item.interface.service_description || '无服务描述',
      match_reason: item.match_reason,
      search_type: item.search_type,
      // 保留原始数据以备后用
      project_id: item.project_id,
      similarity_score: item.similarity_score,
      operation_id: item.interface.operation_id,
      tags: item.interface.tags,
      domain: item.interface.domain,
      deprecated: item.interface.deprecated
    }))
  }
}