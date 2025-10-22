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
import { EndpointsApiService } from '../data/api'
import { toast } from 'sonner'

interface CreateEndpointDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess?: () => void
}

export function CreateEndpointDialog({ open, onOpenChange, onSuccess }: CreateEndpointDialogProps) {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [swaggerContent, setSwaggerContent] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const handleSubmit = async () => {
    if (!name.trim()) {
      toast.error('请输入服务名称')
      return
    }

    if (!swaggerContent.trim()) {
      toast.error('请输入Swagger内容')
      return
    }

    // 验证Swagger内容是否为有效JSON
    try {
      JSON.parse(swaggerContent)
    } catch (error) {
      toast.error('Swagger内容格式不正确，请输入有效的JSON格式')
      return
    }

    setIsLoading(true)
    try {
      await EndpointsApiService.createEndpoint({
        name: name.trim(),
        description: description.trim() || null,
        swagger_content: swaggerContent.trim()
      })
      
      toast.success('端点创建成功')
      
      // 重置表单
      setName('')
      setDescription('')
      setSwaggerContent('')
      
      // 关闭对话框
      onOpenChange(false)
      
      // 通知父组件刷新数据
      if (onSuccess) {
        onSuccess()
      }
    } catch (error) {
      console.error('Failed to create endpoint:', error)
      toast.error('创建端点失败', {
        description: (error as Error).message || '未知错误',
        duration: 10000,
        closeButton: true,
      })
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='max-w-2xl max-h-[90vh] overflow-y-auto'>
        <DialogHeader>
          <DialogTitle>创建端点</DialogTitle>
          <DialogDescription>
            添加一个新的MCP端点服务
          </DialogDescription>
        </DialogHeader>
        <div className='space-y-4 overflow-y-auto'>
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
              className='min-h-[200px] max-h-[400px] overflow-y-auto overflow-x-auto resize-y font-mono text-sm whitespace-nowrap'
            />
          </div>
        </div>
        <div className='flex justify-end space-x-2'>
          <Button variant='outline' onClick={() => onOpenChange(false)} disabled={isLoading}>
            取消
          </Button>
          <Button onClick={handleSubmit} disabled={isLoading}>
            {isLoading ? '创建中...' : '创建'}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}