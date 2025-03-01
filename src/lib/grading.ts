import { type AnalysisResult } from '@/lib/types'

export type DefectLevel = 'MANDATORY' | 'SUGGESTIONS'

// 评分权重常量
export const DEFECT_WEIGHTS: Record<DefectLevel, number> = {
  MANDATORY: 5, // 必须修复的问题扣5分
  SUGGESTIONS: 2, // 建议修复的问题扣2分
}

// 等级定义
export const GRADE_DEFINITIONS = {
  'A+': { minScore: 95, description: '代码质量极其优秀' },
  'A': { minScore: 90, description: '代码质量优秀' },
  'B+': { minScore: 85, description: '代码质量良好' },
  'B': { minScore: 80, description: '代码质量一般' },
  'C': { minScore: 70, description: '代码质量需要改进' },
  'D': { minScore: 0, description: '代码质量亟待提升' },
} as const

export type Grade = keyof typeof GRADE_DEFINITIONS

// 计算分数的函数
export const calculateScore = (results: AnalysisResult[]): { score: number; grade: Grade } => {
  let totalDeduction = 0

  // 计算总扣分
  results.forEach(result => {
    totalDeduction += DEFECT_WEIGHTS[result.defectLevel]
  })

  // 计算最终分数（满分100）
  const score = Math.max(0, 100 - totalDeduction)

  // 确定等级
  let grade: Grade = 'D'
  for (const [g, def] of Object.entries(GRADE_DEFINITIONS)) {
    if (score >= def.minScore) {
      grade = g as Grade
      break
    }
  }

  return { score, grade }
}

export const DefectLevelColor: Record<DefectLevel, string> = {
  MANDATORY: 'bg-red-500',
  SUGGESTIONS: 'bg-yellow-500'
}

export const DefectLevelText: Record<DefectLevel, string> = {
  MANDATORY: '必须修复',
  SUGGESTIONS: '建议修复'
} 