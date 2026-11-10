import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardDescription, CardHeader } from "@/components/ui/card"
import type { Metadata } from "next"

export const metadata: Metadata = {
  alternates: { canonical: "/" },
  openGraph: {
    type: "website",
    locale: "zh_CN",
    siteName: "仓禀",
    title: "仓禀：仓颉代码质量分析与 cjlint 报告",
    description: "分析仓颉 Git 仓库，查看 cjlint 问题分级、质量评分和项目徽章。",
    url: "/",
  },
}

export default function Home() {
  return (
    <main className="flex min-h-[calc(100vh-70px)] flex-col items-center justify-center p-4 md:p-8 overflow-x-hidden">
      <Card className="w-full max-w-[600px] mx-auto">
        <CardHeader>
          <h1 className="text-center text-3xl font-bold">仓禀 · 仓颉代码质量分析</h1>
          <CardDescription className="text-center text-lg mt-2">
            输入 Git 仓库地址，获取 cjlint 代码质量报告
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" action="/result" method="GET">
            <Input 
              aria-label="Git 仓库地址"
              name="repo"
              placeholder="https://github.com/username/repository.git"
              className="text-lg p-6"
            />
            <Button type="submit" className="w-full text-lg p-6">
              分析代码
            </Button>
          </form>
        </CardContent>
      </Card>
      <section className="w-full max-w-[600px] mt-8 text-sm leading-7 text-gray-600" aria-labelledby="report-features">
        <h2 id="report-features" className="text-lg font-semibold text-gray-900">从检查结果定位需要修改的代码</h2>
        <p className="mt-2">仓禀使用 cjlint 检查仓颉项目，按问题级别展示文件位置和说明，并根据检查结果计算质量评分。报告还提供 Markdown 徽章，方便放进项目 README。</p>
        <p className="mt-2">评分反映静态检查发现的问题，不能替代测试和代码审查。分析前请确认仓库可由服务访问。</p>
        <a className="inline-block mt-3 text-blue-600 hover:underline" href="https://github.com/Zxilly/cangjie-card">查看仓禀源码与使用说明</a>
      </section>
    </main>
  )
}
