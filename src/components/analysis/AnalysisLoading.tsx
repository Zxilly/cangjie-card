import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import Link from "next/link"
import { Button } from "@/components/ui/button"

interface AnalysisLoadingProps {
  repo: string
}

export function AnalysisLoading({ repo }: AnalysisLoadingProps) {
  return (
    <main className="flex min-h-screen flex-col items-center p-4 sm:p-6">
      <div className="w-full max-w-7xl">
        <div className="mb-8">
          <div className="text-sm text-gray-600">
            <span className="mr-4">仓库：{repo}</span>
          </div>
        </div>
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