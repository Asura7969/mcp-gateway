export type DatasetType = 'upload' | 'remote'

export interface ColumnSchema {
  name: string
  type: 'string' | 'long' | 'double' | 'datatime'
  description?: string
  searchable?: boolean
  retrievable?: boolean
}

export interface CreateDatasetRequest {
  name: string
  description?: string
  type: DatasetType
  table_name: string
  schema: ColumnSchema[]
  similarity_threshold?: number
  max_results?: number
  retrieval_column?: string
  reply_column?: string
}

export interface UpdateDatasetRequest {
  name?: string
  description?: string
  similarity_threshold?: number
  max_results?: number
  retrieval_column?: string
  reply_column?: string
}

export interface DatasetResponse {
  id: string
  name: string
  description?: string
  type: DatasetType
  table_name: string
  similarity_threshold: number
  max_results: number
}

export interface DatasetDetailResponse {
  id: string
  name: string
  description?: string
  type: DatasetType
  table_name: string
  index_name: string
  table_schema: ColumnSchema[]
  index_mapping?: any
  retrieval_column: string
  reply_column: string
  similarity_threshold: number
  max_results: number
}

export interface IngestRequest {
  dataset_id: string
  file_id: string
}

export interface IngestResult {
  ingested_rows: number
  task_id?: string
}

export type TaskStatus = 'Created' | 'Processing' | 'Completed' | 'Failed'

export interface IngestTask {
  id: string
  dataset_id: string
  file_id: string
  status: TaskStatus
  error?: string | null
  create_time: string
  update_time: string
}

export interface TableSearchRequest {
  dataset_id: string
  query: string
  max_results?: number
  similarity_threshold?: number
}

export interface TableSearchPagedRequest {
  dataset_id: string
  query: string
  page?: number
  page_size?: number
}

export interface PaginationInfo {
  page: number
  page_size: number
  total: number
  total_pages: number
}

export interface PaginatedDatasetsResponse {
  datasets: DatasetResponse[]
  pagination: PaginationInfo
}

export interface EsSearchPagedResponse {
  hits: {
    total: { value: number }
    hits: EsHit[]
  }
  page: number
  page_size: number
  total_pages: number
}

// Elasticsearch response shape (simplified)
export interface EsHitSource {
  page_content: string
  vector: number[]
  row: Record<string, string>
  metadata: { dataset_id: string; table_name: string }
}

export interface EsHit {
  _id: string
  _score: number
  _source: EsHitSource
}

export interface EsSearchResponse {
  hits: {
    total: { value: number }
    hits: EsHit[]
  }
}