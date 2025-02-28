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

type AnalysisResponse = {
  cjlint: AnalysisResult[]
  created_at: number
  commit: string
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
  searchParams: Promise<{ repo: string }>
}) {
  const repo = (await searchParams).repo
  
  const analysisResponse = await redis.get<AnalysisResponse>(`cjlint_${repo}`)
  
  if (!analysisResponse) {
    return (
      <main className="flex min-h-screen flex-col items-center p-4 sm:p-6 pt-24">
        <div className="w-full max-w-7xl">
          <Card>
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
        </div>
      </main>
    )
  }

  const groupedResults = analysisResponse.cjlint.reduce((acc, curr) => {
    if (!acc[curr.defectLevel]) {
      acc[curr.defectLevel] = []
    }
    acc[curr.defectLevel].push(curr)
    return acc
  }, {} as Record<DefectLevel, AnalysisResult[]>)

  const totalIssues = analysisResponse.cjlint.length
  const grade = totalIssues === 0 ? 'A+' : totalIssues <= 5 ? 'A' : totalIssues <= 10 ? 'B' : 'C'

  return (
    <main className="flex min-h-screen flex-col items-center p-4 sm:p-6 pt-24">
      <div className="w-full max-w-7xl">
        <div className="mb-6 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 sm:gap-0">
          <h1 className="text-2xl font-bold">Report for {repo}</h1>
          <div className="flex items-center gap-2 w-full sm:w-auto">
            <Button variant="outline" size="sm" className="h-8 px-3 py-0 flex-1 sm:flex-none justify-center">
              cangjie report A+
            </Button>
            <Button variant="outline" size="sm" className="h-8 px-3 py-0 flex-1 sm:flex-none justify-center">
              Tweet
            </Button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Left Column - Stats */}
          <div className="lg:col-span-3 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-1 gap-6">
            <Card>
              <CardHeader className="pb-3">
                <div className="text-4xl font-bold mb-2">{grade}</div>
                <div className="text-sm text-gray-600">
                  Excellent!
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="text-sm text-gray-500">
                  Found {totalIssues} issues across {analysisResponse.cjlint.length} files
                </div>
              </CardContent>
            </Card>

            <div className="space-y-6">
              <div>
                <h3 className="text-lg font-semibold mb-3">Results</h3>
                <div className="space-y-2">
                  <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
                    <span className="font-medium">cjlint</span>
                    <span className="text-green-600">{totalIssues === 0 ? '93%' : '85%'}</span>
                  </div>
                </div>
              </div>

              <div className="text-sm text-gray-500">
                Last refresh: 4 hours ago
                <Button variant="link" className="text-sm p-0 h-auto ml-2">
                  Refresh now
                </Button>
              </div>
            </div>
          </div>

          {/* Right Column - Analysis Results */}
          <div className="lg:col-span-9">
            <Card>
              <CardHeader>
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
                  <div className="text-lg font-semibold">cjlint</div>
                  <div className="text-sm text-gray-500">
                    提交: {analysisResponse.commit.substring(0, 7)}
                    <span className="hidden sm:inline ml-4">
                      分析时间: {new Date(analysisResponse.created_at * 1000).toLocaleString()}
                    </span>
                    <span className="block sm:hidden">
                      分析时间: {new Date(analysisResponse.created_at * 1000).toLocaleString()}
                    </span>
                  </div>
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