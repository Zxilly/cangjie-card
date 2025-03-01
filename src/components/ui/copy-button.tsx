"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Copy, Check } from "lucide-react"

interface CopyButtonProps {
  text: string
  timeout?: number
  label?: string
  copiedLabel?: string
}

export function CopyButton({ 
  text, 
  timeout = 2000,
  label = "复制Markdown",
  copiedLabel = "已复制"
}: CopyButtonProps) {
  const [copied, setCopied] = useState(false)
  
  const handleCopy = async () => {
    await navigator.clipboard.writeText(text)
    setCopied(true)
    
    // 自动清除copied状态
    const timer = setTimeout(() => setCopied(false), timeout)
    return () => clearTimeout(timer)
  }
  
  return (
    <Button 
      variant="outline" 
      size="sm" 
      onClick={handleCopy}
      className="flex items-center gap-1"
    >
      {copied ? (
        <>
          <Check className="h-4 w-4" />
          {copiedLabel}
        </>
      ) : (
        <>
          <Copy className="h-4 w-4" />
          {label}
        </>
      )}
    </Button>
  )
} 