import { createFileRoute } from '@tanstack/react-router'
import { TableRagPage } from '@/features/table-rag'

export const Route = createFileRoute('/_authenticated/datasets/')({
  component: TableRagPage,
})