import { Suspense } from 'react'
import { LogConsole, LogConsoleRouteAdapter } from '@/components/logs/log-console'

export default function LogsPage() {
  return (
    <Suspense fallback={<LogConsole />}>
      <LogConsoleRouteAdapter />
    </Suspense>
  )
}
