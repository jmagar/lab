'use client'

import { useState, useEffect, useRef } from 'react'
import { ChevronDown, Loader2 } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from 'sonner'
import { installServer } from '@/lib/api/mcpregistry-client'
import { isAbortError } from '@/lib/api/service-action-client'
import { RegistryApiError } from '@/lib/types/registry'
import { logoutBrowserSession } from '@/lib/auth/session'
import { validateGatewayName, deriveGatewayName } from '@/lib/utils/gateway-name'
import { cn } from '@/lib/utils'
import type { ServerJSON } from '@/lib/types/registry'

interface InstallDialogProps {
  server: ServerJSON | null
  onClose: () => void
}


export function InstallDialog({ server, onClose }: InstallDialogProps) {
  const [gatewayName, setGatewayName] = useState('')
  const [bearerTokenEnv, setBearerTokenEnv] = useState('')
  const [advancedOpen, setAdvancedOpen] = useState(false)
  const [nameError, setNameError] = useState<string | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const abortRef = useRef<AbortController | null>(null)

  useEffect(() => {
    if (server) {
      setGatewayName(deriveGatewayName(server.name))
      setBearerTokenEnv('')
      setAdvancedOpen(false)
      setNameError(null)
      setSubmitError(null)
    } else {
      abortRef.current?.abort()
      // Reset derived name when closed so it re-derives on next open
      setGatewayName('')
    }
  }, [server])

  // Abort any in-flight submit on unmount
  useEffect(() => () => { abortRef.current?.abort() }, [])

  const handleGatewayNameChange = (value: string) => {
    setGatewayName(value)
    if (nameError) {
      // Re-validate on change once the user has already seen an error
      setNameError(validateGatewayName(value))
    }
  }

  const handleSubmit = async () => {
    if (!server) return

    const err = validateGatewayName(gatewayName)
    if (err) {
      setNameError(err)
      return
    }
    setNameError(null)
    setSubmitError(null)
    setIsSubmitting(true)

    abortRef.current?.abort()
    abortRef.current = new AbortController()

    try {
      await installServer(
        {
          name: server.name,
          gateway_name: gatewayName,
          version: server.version,
          bearer_token_env: bearerTokenEnv.trim() || undefined,
        },
        abortRef.current.signal,
      )
      toast.success('Server installed successfully')
      onClose()
    } catch (error) {
      if (error instanceof RegistryApiError && error.status === 401) {
        await logoutBrowserSession()
        return
      }
      if (isAbortError(error)) return
      setSubmitError(error instanceof Error ? error.message : 'Failed to install server')
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog open={server !== null} onOpenChange={(open) => { if (!open) onClose() }}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Install to Gateway</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-2">
          <div className="space-y-1.5">
            <Label htmlFor="gateway-name">Gateway name</Label>
            <Input
              id="gateway-name"
              value={gatewayName}
              onChange={(e) => handleGatewayNameChange(e.target.value)}
              placeholder="my-mcp-server"
              disabled={isSubmitting}
            />
            {nameError ? (
              <p className="text-sm text-aurora-error">{nameError}</p>
            ) : (
              <p className="text-xs text-aurora-text-muted">
                Letters, digits, underscores, hyphens — starts with a letter or digit
              </p>
            )}
          </div>

          <div className="rounded-md border border-aurora-border-strong/60 bg-[rgba(14,31,44,0.4)]">
            <button
              type="button"
              onClick={() => setAdvancedOpen((v) => !v)}
              aria-expanded={advancedOpen}
              aria-controls="install-advanced"
              className="flex w-full items-center justify-between px-3 py-2 text-left text-xs font-medium uppercase tracking-[0.14em] text-aurora-text-muted hover:text-aurora-text-primary"
            >
              <span>Advanced</span>
              <ChevronDown
                className={cn('size-3.5 transition-transform', advancedOpen && 'rotate-180')}
              />
            </button>
            {advancedOpen && (
              <div id="install-advanced" className="space-y-1.5 border-t border-aurora-border-strong/40 px-3 py-3">
                <Label htmlFor="bearer-token-env" className="text-xs">
                  Bearer token env var
                </Label>
                <Input
                  id="bearer-token-env"
                  value={bearerTokenEnv}
                  onChange={(e) => setBearerTokenEnv(e.target.value)}
                  placeholder="MY_SERVER_TOKEN"
                  disabled={isSubmitting}
                />
                <p className="text-xs text-aurora-text-muted">
                  Only needed if this server requires a bearer token. Enter the name of an env var
                  (not the token value) that holds it.
                </p>
              </div>
            )}
          </div>

          {submitError && (
            <p className="text-sm text-aurora-error">{submitError}</p>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isSubmitting}>
            Cancel
          </Button>
          <Button onClick={() => void handleSubmit()} disabled={isSubmitting}>
            {isSubmitting && <Loader2 className="size-4 mr-2 animate-spin" />}
            Install to Gateway
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
