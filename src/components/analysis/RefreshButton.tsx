'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { Button } from "@/components/ui/button"
import { RefreshCw } from "lucide-react"

interface RefreshButtonProps {
  repo: string
}

export const RefreshButton = ({ repo }: RefreshButtonProps) => {
  const router = useRouter()
  const [isRefreshing, setIsRefreshing] = useState(false)

  const handleRefresh = async () => {
    try {
      setIsRefreshing(true)
      const response = await fetch(`/api/refresh?repo=${encodeURIComponent(repo)}`)
      if (!response.ok) {
        throw new Error('刷新失败')
      }
      router.refresh()
    } catch (error) {
      console.error('刷新失败:', error)
      alert('刷新失败，请稍后重试')
    } finally {
      setIsRefreshing(false)
    }
  }

  return (
    <Button 
      variant="outline" 
      className="w-full flex items-center justify-center gap-2" 
      onClick={handleRefresh}
      disabled={isRefreshing}
    >
      <RefreshCw className={`h-4 w-4 ${isRefreshing ? 'animate-spin' : ''}`} />
      {isRefreshing ? '正在刷新...' : '刷新分析'}
    </Button>
  )
} 