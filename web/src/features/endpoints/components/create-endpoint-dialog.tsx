import { useState, useEffect } from 'react'
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
import { AxiosError } from 'axios'

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

  // 当对话框关闭时清除表单内容
  useEffect(() => {
    if (!open) {
      setName('')
      setDescription('')
      setSwaggerContent('')
      setIsLoading(false)
    }
  }, [open])

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

      let errorMessage = '未知错误'
      if (error instanceof AxiosError) {
        // 尝试从响应中提取错误信息
        if (error.response?.data) {
          // 如果响应数据是字符串（纯文本错误）
          if (typeof error.response.data === 'string') {
            errorMessage = error.response.data
          } else {
            // 如果响应数据是对象（JSON错误）
            errorMessage = error.response.data.message ||
              error.response.data.error ||
              error.response.data.title ||
              error.message ||
              '未知错误'
          }
        } else {
          errorMessage = error.message || '未知错误'
        }
      } else if (error instanceof Error) {
        errorMessage = error.message
      }

      toast.error('创建端点失败', {
        description: errorMessage,
        duration: 10000,
        closeButton: true,
      })
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className='max-w-4xl max-h-[90vh] overflow-y-auto'
        style={{ width: '90vw', maxWidth: 'none' }}
      >
        <DialogHeader>
          <DialogTitle>创建端点</DialogTitle>
          <DialogDescription>
            添加一个新的MCP端点服务
          </DialogDescription>
        </DialogHeader>
        <div className='space-y-4 overflow-y-auto' style={{ maxWidth: '85vw', width: '85vw' }}>
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
              className='min-h-[200px] max-h-[400px] overflow-y-auto resize-y font-mono text-sm break-words'
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