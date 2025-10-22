import { z } from 'zod'

// Define the endpoint schema based on the backend response
export const endpointSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string().nullable(),
  swagger_content: z.string(),
  status: z.enum(['Running', 'Stopped', 'Deleted']),
  created_at: z.string(),
  updated_at: z.string(),
  connection_count: z.number(),
})

export type Endpoint = z.infer<typeof endpointSchema>

// Schema for creating a new endpoint
export const createEndpointSchema = z.object({
  name: z.string(),
  description: z.string().nullable(),
  swagger_content: z.string(),
})

export type CreateEndpoint = z.infer<typeof createEndpointSchema>

// Schema for updating an endpoint
export const updateEndpointSchema = endpointSchema.partial().omit({
  id: true,
  created_at: true,
  connection_count: true,
})

export type UpdateEndpoint = z.infer<typeof updateEndpointSchema>

// Schema for endpoint detail response
export const endpointDetailSchema = endpointSchema.extend({
  swagger_spec: z.any().nullable(),
  mcp_config: z.any().nullable(),
  api_details: z.array(z.any()).nullable(),
  base_url: z.string().nullable(),
})

export type EndpointDetail = z.infer<typeof endpointDetailSchema>