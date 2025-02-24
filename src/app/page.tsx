import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useState } from "react"
import { useRouter } from "next/navigation"

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-24">
      <Card className="w-[600px]">
        <CardHeader>
          <CardTitle className="text-center text-3xl font-bold">仓颉代码质量检查</CardTitle>
          <CardDescription className="text-center text-lg mt-2">
            输入Git仓库地址，获取代码质量报告
          </CardDescription>
        </CardHeader>
        <CardContent>
          <form className="flex flex-col gap-4" action="/result" method="GET">
            <Input 
              name="repo"
              placeholder="https://github.com/username/repository"
              className="text-lg p-6"
            />
            <Button type="submit" className="w-full text-lg p-6">
              分析代码
            </Button>
          </form>
        </CardContent>
      </Card>
    </main>
  )
}
