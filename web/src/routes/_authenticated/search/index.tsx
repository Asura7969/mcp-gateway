import { createFileRoute } from '@tanstack/react-router'
import { SearchDebug } from '@/features/search'

export const Route = createFileRoute('/_authenticated/search/')({
  component: SearchDebug,
})