import { generateBadge } from '@/lib/badge';
import { redis } from '@/lib/db';
import { calculateScore } from '@/lib/grading';
import { type AnalysisResponse } from '@/lib/types';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  const searchParams = request.nextUrl.searchParams;
  const repo = searchParams.get('repo');

  if (!repo) {
    return NextResponse.json(
      { error: '需要提供repo参数' },
      { status: 400 }
    );
  }

  try {
    const analysisResponse = await redis.get<AnalysisResponse>(`cjlint_${repo}`);
    
    if (!analysisResponse) {
      return NextResponse.json(
        { error: '未找到该仓库的分析结果' },
        { status: 404 }
      );
    }
    
    const { score, grade } = calculateScore(analysisResponse.cjlint);
    const svg = generateBadge(grade, score);
    
    return new NextResponse(svg, {
      headers: {
        'Content-Type': 'image/svg+xml',
        'Cache-Control': 'public, max-age=3600',
      },
    });
  } catch (error) {
    console.error('生成徽章时出错:', error);
    return NextResponse.json(
      { error: '生成徽章时出错' },
      { status: 500 }
    );
  }
} 