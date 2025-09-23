import axios from 'axios'
import { type Endpoint, type CreateEndpoint, type UpdateEndpoint, type EndpointDetail } from './schema'

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000',
  timeout: 10000,
})

export class EndpointsApiService {
  static async getEndpoints(): Promise<Endpoint[]> {
    const response = await api.get('/api/endpoint')
    return response.data
  }

  static async getEndpointsPaginated(
    page?: number,
    pageSize?: number,
    search?: string,
    status?: string
  ): Promise<{ endpoints: Endpoint[]; pagination: any }> {
    const params: any = {}
    if (page) params.page = page
    if (pageSize) params.page_size = pageSize
    if (search) params.search = search
    if (status) params.status = status

    const response = await api.get('/api/endpoints', { params })
    return response.data
  }

  static async getEndpointById(id: string): Promise<EndpointDetail> {
    const response = await api.get(`/api/endpoint/${id}`)
    return response.data
  }

  static async createEndpoint(endpoint: CreateEndpoint): Promise<Endpoint> {
    const response = await api.post('/api/endpoint', endpoint)
    return response.data
  }

  static async updateEndpoint(id: string, endpoint: UpdateEndpoint): Promise<Endpoint> {
    const response = await api.put(`/api/endpoint/${id}`, endpoint)
    return response.data
  }

  static async deleteEndpoint(id: string): Promise<void> {
    await api.delete(`/api/endpoint/${id}`)
  }
}