import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Redis } from '@upstash/redis'
import { AnalysisLoading } from "@/components/analysis/AnalysisLoading"
import { RefreshButton } from "@/components/analysis/RefreshButton"

const redis = new Redis({
  url: process.env.KV_REST_API_URL!,
  token: process.env.KV_REST_API_READ_ONLY_TOKEN!,
})

type DefectLevel = 'MANDATORY' | 'SUGGESTIONS'

// 添加评分权重常量
const DEFECT_WEIGHTS: Record<DefectLevel, number> = {
  MANDATORY: 10, // 必须修复的问题扣10分
  SUGGESTIONS: 3, // 建议修复的问题扣3分
}

// 添加等级定义
const GRADE_DEFINITIONS = {
  'A+': { minScore: 95, description: '代码质量极其优秀' },
  'A': { minScore: 90, description: '代码质量优秀' },
  'B+': { minScore: 85, description: '代码质量良好' },
  'B': { minScore: 80, description: '代码质量一般' },
  'C': { minScore: 70, description: '代码质量需要改进' },
  'D': { minScore: 0, description: '代码质量亟待提升' },
} as const

type Grade = keyof typeof GRADE_DEFINITIONS

// 添加计算分数的函数
const calculateScore = (results: AnalysisResult[]): { score: number; grade: Grade } => {
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

type AnalysisResult = {
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

type AnalysisResponse = {
  cjlint: AnalysisResult[]
  created_at: number
  commit: string
  package_name: string
}

const DefectLevelColor: Record<DefectLevel, string> = {
  MANDATORY: 'bg-red-500',
  SUGGESTIONS: 'bg-yellow-500'
}

const DefectLevelText: Record<DefectLevel, string> = {
  MANDATORY: '必须修复',
  SUGGESTIONS: '建议修复'
}

// 添加格式化相对时间的函数
const formatRelativeTime = (timestamp: number): string => {
  const now = Math.floor(Date.now() / 1000)
  const diff = now - timestamp
  
  if (diff < 60) {
    return '刚刚'
  } else if (diff < 3600) {
    const minutes = Math.floor(diff / 60)
    return `${minutes}分钟前`
  } else if (diff < 86400) {
    const hours = Math.floor(diff / 3600)
    return `${hours}小时前`
  } else if (diff < 2592000) {
    const days = Math.floor(diff / 86400)
    return `${days}天前`
  } else if (diff < 31536000) {
    const months = Math.floor(diff / 2592000)
    return `${months}个月前`
  } else {
    const years = Math.floor(diff / 31536000)
    return `${years}年前`
  }
}

export default async function ResultPage({
  searchParams,
}: {
  searchParams: Promise<{ repo: string }>
}) {
  const repo = (await searchParams).repo
  
  const analysisResponse = await redis.get<AnalysisResponse>(`cjlint_${repo}`)
  
  if (!analysisResponse) {
    return <AnalysisLoading repo={repo} />
  }

  const package_name = analysisResponse.package_name;

  const groupedResults = analysisResponse.cjlint.reduce((acc, curr) => {
    if (!acc[curr.defectLevel]) {
      acc[curr.defectLevel] = []
    }
    acc[curr.defectLevel].push(curr)
    return acc
  }, {} as Record<DefectLevel, AnalysisResult[]>)

  const { score, grade } = calculateScore(analysisResponse.cjlint)
  const gradeDescription = GRADE_DEFINITIONS[grade].description
  const totalIssues = analysisResponse.cjlint.length

  return (
    <main className="flex min-h-screen flex-col items-center p-4 sm:p-6">
      <div className="w-full max-w-7xl">
        <div className="mb-8">
          <div className="flex justify-between items-start">
            <h1 className="text-4xl font-bold">{package_name}</h1>
            <div className="text-sm text-gray-600 space-y-1 text-right">
              <div>仓库：{repo}</div>
              <div>Commit：{analysisResponse.commit}</div>
            </div>
          </div>
        </div>
        <div className="mb-6 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 sm:gap-0">
          <div className="flex items-center gap-2 w-full sm:w-auto">
            <Button variant="outline" size="sm" className="h-8 px-3 py-0 flex-1 sm:flex-none justify-center">
              Cangjie 分析报告 A+
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-3">
            <Card className="mb-6">
              <CardHeader className="pb-3">
                <div className="text-4xl font-bold mb-2">{grade}</div>
                <div className="text-sm text-gray-600">
                  {gradeDescription}
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="text-sm text-gray-500">
                  在分析中发现 {totalIssues} 个问题，得分：{score.toFixed(1)}
                </div>
              </CardContent>
            </Card>

            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold mb-3">分析结果</h3>
                <div className="space-y-2">
                  <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
                    <span className="font-medium">cjlint</span>
                    <span className="text-green-600">{score.toFixed(1)}%</span>
                  </div>
                </div>
              </div>

              <div className="space-y-2">
                <div className="text-sm text-gray-500">
                  上次更新：{formatRelativeTime(analysisResponse.created_at)}
                </div>
                <RefreshButton repo={repo} />
              </div>
            </div>
          </div>

          <div className="lg:col-span-9">
            <Card>
              <CardHeader>
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                  <div className="text-lg font-semibold">cjlint</div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-6">
                  {Object.entries(groupedResults).map(([level, issues]) => (
                    <div key={level} className="border rounded-lg p-4">
                      <div className="flex items-center gap-2 mb-4">
                        <div className={`w-2 h-2 rounded-full ${DefectLevelColor[level as DefectLevel]}`}></div>
                        <h3 className="text-xl font-semibold">
                          {DefectLevelText[level as DefectLevel]} ({issues.length})
                        </h3>
                      </div>
                      <div className="space-y-4">
                        {issues.map((issue, index) => (
                          <div key={index} className="bg-gray-50 rounded p-3">
                            <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-2 sm:gap-0">
                              <div className="text-sm text-gray-600 break-all">
                                {issue.file}:{issue.line}:{issue.column}
                              </div>
                              <div className="text-sm font-mono bg-gray-200 px-2 py-0.5 rounded self-start">
                                {issue.language}
                              </div>
                            </div>
                            <p className="mt-2">{issue.description}</p>
                            <div className="mt-1 text-sm text-gray-500">
                              类型: {issue.defectType}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </main>
  )
}