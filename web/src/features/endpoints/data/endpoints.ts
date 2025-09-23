import { faker } from '@faker-js/faker'
import { type Endpoint } from './schema'

// Set a fixed seed for consistent data generation
faker.seed(12345)

export const endpoints = Array.from({ length: 20 }, () => {
  const statuses = ['Running', 'Stopped', 'Deleted'] as const
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_protocols] = [['stdio', 'sse', 'streamable'] as const]

  const methodCount = faker.number.int({ min: 1, max: 10 })
  const connectionCount = faker.number.int({ min: 0, max: 100 })

  return {
    id: `ENDPOINT-${faker.number.int({ min: 1000, max: 9999 })}`,
    name: faker.company.name(),
    description: faker.lorem.sentence(),
    swagger_content: JSON.stringify({
      openapi: '3.0.0',
      info: {
        title: faker.company.name(),
        version: '1.0.0',
      },
      paths: {
        '/api/test': {
          get: {
            summary: 'Test endpoint',
            responses: {
              '200': {
                description: 'Successful response',
              },
            },
          },
        },
      },
    }),
    status: faker.helpers.arrayElement(statuses),
    created_at: faker.date.past().toISOString(),
    updated_at: faker.date.recent().toISOString(),
    connection_count: connectionCount,
    base_url: `https://${faker.internet.domainName()}/api`,
    method_count: methodCount,
  } as Endpoint
})