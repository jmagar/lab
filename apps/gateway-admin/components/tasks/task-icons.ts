import { Bug, BookOpen, CheckSquare, Settings, Zap } from 'lucide-react'
import type { IssueType } from '@/lib/types/beads'

/** Map each issue type to its lucide icon component. */
export const TYPE_ICON: Record<IssueType, typeof Bug> = {
  task: CheckSquare,
  epic: BookOpen,
  bug: Bug,
  feature: Zap,
  chore: Settings,
}
