'use client'

import { CheckCircle2, XCircle, Clock, Wrench, FileText, MessageSquare, X } from 'lucide-react'
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import type { Gateway, TestGatewayResult } from '@/lib/types/gateway'

interface TestResultPanelProps {
  result: { gateway: Gateway; result: TestGatewayResult } | null
  onClose: () => void
}

export function TestResultPanel({ result, onClose }: TestResultPanelProps) {
  if (!result) return null

  const { gateway, result: testResult } = result

  return (
    <Sheet open={!!result} onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="sm:max-w-md">
        <SheetHeader>
          <SheetTitle>Connection Test Results</SheetTitle>
          <SheetDescription>
            Test results for {gateway.name}
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          {/* Status */}
          <div className={`flex items-start gap-4 rounded-lg border p-4 ${
            testResult.success 
              ? 'border-emerald-500/20 bg-emerald-500/5' 
              : 'border-red-500/20 bg-red-500/5'
          }`}>
            {testResult.success ? (
              <CheckCircle2 className="size-5 text-emerald-600 dark:text-emerald-400 mt-0.5" />
            ) : (
              <XCircle className="size-5 text-red-600 dark:text-red-400 mt-0.5" />
            )}
            <div className="flex-1">
              <p className={`font-medium ${
                testResult.success 
                  ? 'text-emerald-600 dark:text-emerald-400' 
                  : 'text-red-600 dark:text-red-400'
              }`}>
                {testResult.success ? 'Connection Successful' : 'Connection Failed'}
              </p>
              <p className="text-sm text-muted-foreground mt-0.5">
                {testResult.message}
              </p>
              {testResult.error && (
                <p className="text-sm text-red-600 dark:text-red-400 mt-2 font-mono bg-red-500/10 rounded px-2 py-1">
                  {testResult.error}
                </p>
              )}
            </div>
          </div>

          {/* Metrics */}
          {testResult.success && (
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-muted-foreground">Connection Details</h4>
              
              <div className="grid gap-3">
                {testResult.latency_ms !== undefined && (
                  <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                    <div className="flex items-center gap-2 text-sm">
                      <Clock className="size-4 text-muted-foreground" />
                      <span>Latency</span>
                    </div>
                    <span className="text-sm font-medium tabular-nums">
                      {testResult.latency_ms}ms
                    </span>
                  </div>
                )}

                {testResult.discovered_tools !== undefined && (
                  <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                    <div className="flex items-center gap-2 text-sm">
                      <Wrench className="size-4 text-muted-foreground" />
                      <span>Discovered Tools</span>
                    </div>
                    <span className="text-sm font-medium tabular-nums">
                      {testResult.discovered_tools}
                    </span>
                  </div>
                )}

                {testResult.discovered_resources !== undefined && (
                  <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                    <div className="flex items-center gap-2 text-sm">
                      <FileText className="size-4 text-muted-foreground" />
                      <span>Discovered Resources</span>
                    </div>
                    <span className="text-sm font-medium tabular-nums">
                      {testResult.discovered_resources}
                    </span>
                  </div>
                )}

                {testResult.discovered_prompts !== undefined && (
                  <div className="flex items-center justify-between rounded-lg border px-4 py-3">
                    <div className="flex items-center gap-2 text-sm">
                      <MessageSquare className="size-4 text-muted-foreground" />
                      <span>Discovered Prompts</span>
                    </div>
                    <span className="text-sm font-medium tabular-nums">
                      {testResult.discovered_prompts}
                    </span>
                  </div>
                )}
              </div>
            </div>
          )}
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
