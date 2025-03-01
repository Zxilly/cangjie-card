import { type DefectLevel } from './grading'

export type AnalysisResult = {
  file: string
  line: number
  column: number
  endLine: number
  endColumn: number
  analyzerName: string
  description: string
  defectLevel: DefectLevel
  defectType: string
  language: string
}

export type AnalysisResponse = {
  cjlint: AnalysisResult[]
  created_at: number
  commit: string
  package_name: string
}

// 添加格式化相对时间的函数
export const formatRelativeTime = (timestamp: number): string => {
  const now = Math.floor(Date.now() / 1000)
  const diff = now - timestamp
  
  if (diff < 60) {
    return '刚刚'
  } else if (diff < 3600) {
    const minutes = Math.floor(diff / 60)
    return `${minutes} 分钟前`
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600)
    return `${hours} 小时前`
  } else if (diff < 2592000) {
    const days = Math.floor(diff / 86400)
    return `${days} 天前`
  } else if (diff < 31536000) {
    const months = Math.floor(diff / 2592000)
    return `${months} 个月前`
  } else {
    const years = Math.floor(diff / 31536000)
    return `${years} 年前`
  }
} 