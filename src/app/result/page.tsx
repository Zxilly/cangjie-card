import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import Link from "next/link"
import { Button } from "@/components/ui/button"
import { Redis } from '@upstash/redis'

const redis = new Redis({
  url: process.env.KV_REST_API_URL!,
  token: process.env.KV_REST_API_READ_ONLY_TOKEN!,
})

type DefectLevel = 'MANDATORY' | 'SUGGESTIONS'
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

const DefectLevelColor: Record<DefectLevel, string> = {
  MANDATORY: 'bg-red-500',
  SUGGESTIONS: 'bg-yellow-500'
}

const DefectLevelText: Record<DefectLevel, string> = {
  MANDATORY: '必须修复',
  SUGGESTIONS: '建议修复'
}

export default async function ResultPage({
  searchParams,
}: {
  searchParams: { repo: string }
}) {
  const repo = searchParams.repo
  
  const analysisResult = await redis.get(`cangjie_card_${repo}`)
  
  if (!analysisResult) {
    return (
      <main className="flex min-h-screen flex-col items-center p-24">
        <Card className="w-[800px]">
          <CardHeader>
            <CardTitle className="text-2xl font-bold">暂无分析结果</CardTitle>
            <CardDescription className="text-lg">
              仓库地址: {repo}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="text-center py-8">
              <p className="text-gray-600 mb-4">正在分析中，请稍后刷新页面...</p>
              <Link href="/">
                <Button variant="outline">返回首页</Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </main>
    )
  }

  const results = JSON.parse(analysisResult as string) as AnalysisResult[]

  const groupedResults = results.reduce((acc, curr) => {
    if (!acc[curr.defectLevel]) {
      acc[curr.defectLevel] = []
    }
    acc[curr.defectLevel].push(curr)
    return acc
  }, {} as Record<DefectLevel, AnalysisResult[]>)

  return (
    <main className="flex min-h-screen flex-col items-center p-24">
      <Card className="w-[800px]">
        <CardHeader>
          <CardTitle className="text-2xl font-bold">分析结果</CardTitle>
          <CardDescription className="text-lg">
            仓库地址: {repo}
          </CardDescription>
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
                      <div className="flex items-start justify-between">
                        <div className="text-sm text-gray-600">
                          {issue.file}:{issue.line}:{issue.column}
                        </div>
                        <div className="text-sm font-mono bg-gray-200 px-2 py-0.5 rounded">
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

            <div className="mt-8 flex justify-center">
              <Link href="/">
                <Button variant="outline">返回首页</Button>
              </Link>
            </div>
          </div>
        </CardContent>
      </Card>
    </main>
  )
} 