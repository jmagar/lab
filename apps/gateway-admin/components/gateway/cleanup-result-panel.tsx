'use client'

import { ShieldAlert, Wrench, X } from 'lucide-react'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import type { Gateway, GatewayCleanupResult } from '@/lib/types/gateway'

interface CleanupResultPanelProps {
  result: { gateway: Gateway; result: GatewayCleanupResult } | null
  onClose: () => void
}

export function CleanupResultPanel({ result, onClose }: CleanupResultPanelProps) {
  if (!result) return null

  const { gateway, result: cleanup } = result
  const totalKilled =
    cleanup.gateway_killed + cleanup.local_killed + cleanup.aggressive_killed

  return (
    <Sheet open={!!result} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="sm:max-w-md">
        <SheetHeader>
          <SheetTitle>Cleanup Results</SheetTitle>
          <SheetDescription>
            Runtime cleanup results for {gateway.name}
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div
            className={`flex items-start gap-4 rounded-lg border p-4 ${
              cleanup.aggressive
                ? 'border-aurora-warn/20 bg-aurora-warn/5'
                : 'border-aurora-success/20 bg-aurora-success/5'
            }`}
          >
            {cleanup.aggressive ? (
              <ShieldAlert className="size-5 text-aurora-warn mt-0.5" />
            ) : (
              <Wrench className="size-5 text-aurora-success mt-0.5" />
            )}
            <div className="flex-1">
              <p
                className={`font-medium ${
                  cleanup.aggressive ? 'text-aurora-warn' : 'text-aurora-success'
                }`}
              >
                {cleanup.aggressive ? 'Aggressive cleanup completed' : 'Runtime cleanup completed'}
              </p>
              <p className="text-sm text-aurora-text-muted mt-0.5">
                {totalKilled} process{totalKilled === 1 ? '' : 'es'} matched and terminated.
              </p>
              <p className="mt-2 text-xs text-aurora-text-muted">
                Gateway-side tracked matches, local leaked session workers, and the aggressive fallback lane are reported separately below.
              </p>
            </div>
          </div>

          <div className="space-y-3">
            <h4 className="text-sm font-medium text-aurora-text-muted">Cleanup breakdown</h4>
            <div className="grid gap-3">
              <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                <span className="text-sm">Gateway runtime matches</span>
                <span className="text-sm font-medium tabular-nums">{cleanup.gateway_killed}</span>
              </div>
              <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                <span className="text-sm">Local client/session matches</span>
                <span className="text-sm font-medium tabular-nums">{cleanup.local_killed}</span>
              </div>
              {cleanup.aggressive && (
                <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                  <span className="text-sm">Aggressive fallback matches</span>
                  <span className="text-sm font-medium tabular-nums">{cleanup.aggressive_killed}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        <div className="mt-8">
          <Button variant="outline" onClick={onClose} className="w-full">
            <X className="size-4 mr-2" />
            Close
          </Button>
        </div>
      </SheetContent>
    </Sheet>
  )
}
