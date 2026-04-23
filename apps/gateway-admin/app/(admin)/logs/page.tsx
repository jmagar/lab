import { Suspense } from 'react'

import { LogConsole } from '@/components/logs/log-console'
import { Skeleton } from '@/components/ui/skeleton'

export default function LogsPage() {
  return (
    <Suspense
      fallback={(
        <div className="flex-1 p-6">
          <div className="space-y-4">
            <Skeleton className="h-10 w-64" />
            <Skeleton className="h-[480px] w-full rounded-lg" />
          </div>
        </div>
      )}
    >
      <LogConsole />
    </Suspense>
  )
}
