import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { AnalysisLoading } from "@/components/analysis/AnalysisLoading"
import { RefreshButton } from "@/components/analysis/RefreshButton"
import { Badge } from "@/components/analysis/Badge"
import { CopyMarkdownButton } from "@/components/ui/copy-markdown-button"
import { redis } from "@/lib/db"
import { calculateScore, DefectLevelColor, DefectLevelText, type DefectLevel, GRADE_DEFINITIONS } from "@/lib/grading"
import { formatRelativeTime, type AnalysisResponse, type AnalysisResult } from "@/lib/types"
import { Folder, Clipboard } from "lucide-react"
import Link from "next/link"

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

  const packageName = analysisResponse.package_name
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
    <main className="flex flex-col items-center p-4 sm:p-6">
      <div className="w-full max-w-7xl">
        <div className="mb-8">
          <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
            <h1 className="text-3xl sm:text-4xl font-extrabold bg-gradient-to-r from-[#2d76ee] to-[#2ee89f] text-transparent bg-clip-text leading-relaxed py-2">
              {packageName}
            </h1>
            <div className="bg-gray-50 rounded-lg p-3 border border-gray-100 shadow-sm w-full sm:w-auto">
              <div className="flex items-center gap-2 mb-2">
                <Folder className="h-4 w-4 text-gray-500" />
                <div className="text-sm font-medium text-gray-700">
                  仓库：
                  <Link 
                    href={`${repo.replace(/\.git$/, '')}`} 
                    target="_blank" 
                    rel="noopener noreferrer" 
                    className="text-blue-600 hover:underline"
                  >
                    {repo.replace(/\.git$/, '')}
                  </Link>
                </div>
              </div>
              <div className="flex items-center gap-2">
                <Clipboard className="h-4 w-4 text-gray-500" />
                <div className="text-sm font-medium text-gray-700">
                  Commit：<span className="font-mono text-gray-600">{analysisResponse.commit.substring(0, 7)}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          <div className="lg:col-span-3 space-y-6">
            <Card>
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

            <Card>
              <CardHeader className="pb-3">
                <h3 className="text-lg font-semibold">项目徽章</h3>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="flex flex-col items-start gap-3">
                  <Badge grade={grade} score={score} />
                  <CopyMarkdownButton repo={repo} />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <h3 className="text-lg font-semibold">分析结果</h3>
              </CardHeader>
              <CardContent className="pt-0 space-y-4">
                <div className="space-y-2">
                  <div className="flex items-center justify-between p-2 bg-gray-50 rounded">
                    <span className="font-medium">cjlint</span>
                    <span className="text-green-600">{score.toFixed(1)}%</span>
                  </div>
                </div>

                <div className="space-y-2 pt-2 border-t border-gray-100">
                  <div className="text-sm text-gray-500">
                    上次更新：{formatRelativeTime(analysisResponse.created_at)}
                  </div>
                  <RefreshButton repo={repo} />
                </div>
              </CardContent>
            </Card>
          </div>

          <div className="lg:col-span-9">
            <Card>
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <h2 className="text-xl font-semibold">cjlint 详细分析</h2>
                </div>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="space-y-6">
                  {Object.entries(groupedResults).map(([level, issues]) => (
                    <div key={level} className="border rounded-lg p-4">
                      <div className="flex items-center gap-2 mb-4">
                        <div className={`w-3 h-3 rounded-full ${DefectLevelColor[level as DefectLevel]}`}></div>
                        <h3 className="text-xl font-semibold">
                          {DefectLevelText[level as DefectLevel]} ({issues.length})
                        </h3>
                      </div>
                      <div className="space-y-4">
                        {issues.map((issue, index) => (
                          <div key={index} className="bg-gray-50 rounded-lg p-4 border border-gray-100">
                            <div className="flex flex-col sm:flex-row sm:items-start justify-between gap-2">
                              <div className="text-sm text-gray-600 break-all">
                                {issue.file.replace("/tmp/cjrepo/", "")}:{issue.line}:{issue.column}
                              </div>
                              <div className="text-sm font-mono bg-gray-200 px-2 py-0.5 rounded self-start">
                                {issue.language}
                              </div>
                            </div>
                            <p className="mt-2 text-gray-800">{issue.description}</p>
                            <div className="mt-2 text-sm text-gray-500">
                              类型: <span className="font-medium">{issue.defectType}</span>
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