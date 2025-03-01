import { type Grade } from '@/lib/grading';
import { generateBadge } from '@/lib/badge';

interface BadgeProps {
  grade: Grade;
  score: number;
}

export function Badge({ grade, score }: BadgeProps) {  
  return (
    <div 
      className="inline-block" 
      dangerouslySetInnerHTML={{ 
        __html: generateBadge(grade, score) 
      }}
    />
  );
} 