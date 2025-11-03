import axios from 'axios'
import {
  type CreateDatasetRequest,
  type UpdateDatasetRequest,
  type DatasetResponse,
  type IngestRequest,
  type IngestResult,
  type TableSearchRequest,
  type EsSearchResponse,
  type ColumnSchema,
  type IngestTask,
} from './schema'

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
  timeout: 15000,
})

export class TableRagApiService {
  static async listDatasets(page?: number, page_size?: number): Promise<DatasetResponse[]> {
    const params: Record<string, any> = {}
    if (page) params.page = page
    if (page_size) params.page_size = page_size
    const res = await api.get('/api/table-rag/datasets', { params })
    return res.data
  }

  static async createDataset(payload: CreateDatasetRequest): Promise<DatasetResponse> {
    const res = await api.post('/api/table-rag/datasets', payload)
    return res.data
  }

  static async getDataset(id: string): Promise<import('./schema').DatasetDetailResponse> {
    const res = await api.get(`/api/table-rag/datasets/${id}`)
    return res.data
  }

  static async updateDataset(id: string, payload: UpdateDatasetRequest): Promise<DatasetResponse> {
    const res = await api.put(`/api/table-rag/datasets/${id}`, payload)
    return res.data
  }

  static async ingestFile(payload: IngestRequest): Promise<IngestResult> {
    const res = await api.post('/api/table-rag/ingest', payload)
    return res.data
  }

  static async search(payload: TableSearchRequest): Promise<EsSearchResponse> {
    const res = await api.post('/api/table-rag/search', payload)
    return res.data
  }

  static async previewSchema(fileIds: string[]): Promise<ColumnSchema[]> {
    const res = await api.post('/api/table-rag/preview-schema', { file_ids: fileIds })
    return res.data
  }

  static async listTasks(datasetId: string, page?: number, page_size?: number): Promise<IngestTask[]> {
    const params: Record<string, any> = { dataset_id: datasetId }
    if (page) params.page = page
    if (page_size) params.page_size = page_size
    const res = await api.get('/api/table-rag/tasks', { params })
    return res.data
  }
}