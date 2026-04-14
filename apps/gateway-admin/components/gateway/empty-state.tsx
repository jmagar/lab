import { Cable, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface EmptyStateProps {
  title: string
  description: string
  action?: {
    label: string
    onClick: () => void
  }
  icon?: React.ReactNode
  className?: string
}

export function EmptyState({ 
  title, 
  description, 
  action, 
  icon,
  className 
}: EmptyStateProps) {
  return (
    <div className={cn('flex flex-col items-center justify-center py-16 px-4', className)}>
      <div className="flex size-12 items-center justify-center rounded-full bg-muted">
        {icon || <Cable className="size-6 text-muted-foreground" />}
      </div>
      <h3 className="mt-4 text-lg font-medium">{title}</h3>
      <p className="mt-1 text-sm text-muted-foreground text-center max-w-sm">
        {description}
      </p>
      {action && (
        <Button onClick={action.onClick} className="mt-6">
          <Plus className="size-4 mr-2" />
          {action.label}
        </Button>
      )}
    </div>
  )
}
