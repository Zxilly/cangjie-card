"use client"

import { useState, useEffect } from "react"
import { CopyButton } from "@/components/ui/copy-button"

interface CopyMarkdownButtonProps {
  repo: string
}

export function CopyMarkdownButton({ repo }: CopyMarkdownButtonProps) {
  const [markdownBadge, setMarkdownBadge] = useState("")
  
  useEffect(() => {
    // 在客户端动态获取当前host
    const host = window.location.host
    const protocol = window.location.protocol
    
    const badgeUrl = `${protocol}//${host}/badge?repo=${repo}`
    const resultUrl = `${protocol}//${host}/result?repo=${repo}`
    
    setMarkdownBadge(`[![Cangjie](${badgeUrl})](${resultUrl})`)
  }, [repo])
  
  return <CopyButton text={markdownBadge} />
} 