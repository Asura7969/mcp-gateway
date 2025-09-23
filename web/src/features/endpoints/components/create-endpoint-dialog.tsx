import { useState } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'

interface CreateEndpointDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateEndpointDialog({ open, onOpenChange }: CreateEndpointDialogProps) {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [swaggerContent, setSwaggerContent] = useState('')

  const handleSubmit = () => {
    // Handle form submission
    console.log({ name, description, swaggerContent })
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='max-w-2xl'>
        <DialogHeader>
          <DialogTitle>创建端点</DialogTitle>
          <DialogDescription>
            添加一个新的MCP端点服务
          </DialogDescription>
        </DialogHeader>
        <div className='space-y-4'>
          <div className='space-y-2'>
            <Label htmlFor='name'>服务名称</Label>
            <Input
              id='name'
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder='请输入服务名称'
            />
          </div>
          <div className='space-y-2'>
            <Label htmlFor='description'>描述</Label>
            <Input
              id='description'
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder='请输入描述'
            />
          </div>
          <div className='space-y-2'>
            <Label htmlFor='swagger'>Swagger内容</Label>
            <Textarea
              id='swagger'
              value={swaggerContent}
              onChange={(e) => setSwaggerContent(e.target.value)}
              placeholder='请输入Swagger JSON或YAML内容'
              className='min-h-[200px]'
            />
          </div>
        </div>
        <div className='flex justify-end space-x-2'>
          <Button variant='outline' onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button onClick={handleSubmit}>创建</Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}