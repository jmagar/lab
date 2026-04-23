import { Suspense } from 'react'
import { LogConsoleRouteAdapter, LogConsoleSkeleton } from '@/components/logs/log-console'

export default function LogsPage() {
  return (
    <Suspense fallback={<LogConsoleSkeleton />}>
      <LogConsoleRouteAdapter />
    </Suspense>
  )
}
