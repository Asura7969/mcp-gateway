export type UploadedFileMeta = {
  id: string
  type: string
  name?: string
  path: string
  size?: number
  create_time?: string
  update_time?: string
}

export async function uploadFiles(files: File[]): Promise<UploadedFileMeta[]> {
  const baseUrl = (import.meta as any).env?.VITE_API_BASE_URL || 'http://localhost:3000'
  const form = new FormData()
  for (const f of files) {
    form.append('files', f, f.name)
  }
  const res = await fetch(`${baseUrl}/api/files/upload`, {
    method: 'POST',
    body: form,
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || 'Upload failed')
  }
  const data = await res.json()
  return (data.files || []) as UploadedFileMeta[]
}