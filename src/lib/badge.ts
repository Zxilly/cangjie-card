import { makeBadge } from 'badge-maker';
import { type Grade } from './grading';

interface BadgeOptions {
  label: string;
  message: string;
  color: string;
  style?: 'flat' | 'flat-square' | 'plastic' | 'social' | 'for-the-badge';
}

const GRADE_COLORS: Record<Grade, string> = {
  'A+': 'brightgreen',
  'A': 'green',
  'B+': 'yellowgreen',
  'B': 'yellow',
  'C': 'orange',
  'D': 'red',
};

// 生成SVG字符串的函数
export function generateBadge(grade: Grade, score: number): string {
  const options: BadgeOptions = {
    label: 'Cangjie',
    message: `${grade} ${score.toFixed(1)}`,
    color: GRADE_COLORS[grade],
    style: 'flat',
  };
  
  return makeBadge(options);
} 