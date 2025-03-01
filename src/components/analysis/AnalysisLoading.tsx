'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import Link from "next/link"
import { Button } from "@/components/ui/button"
import { useEffect, useState } from "react"
import { useRouter } from "next/navigation"
import { AlertTriangle, RefreshCw } from "lucide-react"

interface AnalysisLoadingProps {
  repo: string
}

interface ApiError {
  message: string;
}

export function AnalysisLoading({ repo }: AnalysisLoadingProps) {
  const router = useRouter()
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const checkAnalysisStatus = async () => {
      try {
        const response = await fetch(`/api/refresh?repo=${encodeURIComponent(repo)}`)
        
        if (response.ok) {
          router.refresh()
          return
        } else {
          const errorData = await response.json() as ApiError;
          setError(errorData.message || "未知错误");
        }
      } catch (err) {
        setError(err instanceof Error ? err.message : "网络错误");
      }
    }

    checkAnalysisStatus()
  }, [repo, router])

  return (
    <main className="flex min-h-screen flex-col items-center p-4 sm:p-6">
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
              {error ? (
                <div className="mb-6">
                  <div className="flex justify-center mb-4">
                    <AlertTriangle className="w-12 h-12 text-red-500" />
                  </div>
                  <p className="text-red-500 mb-4 text-lg">{error}</p>
                </div>
              ) : (
                <>
                  <div className="flex justify-center mb-6">
                    <RefreshCw className="w-12 h-12 text-blue-500 animate-spin" />
                  </div>
                  <p className="text-gray-600 mb-4 text-lg">
                    正在分析中，系统将自动刷新...
                  </p>
                </>
              )}
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