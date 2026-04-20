'use client'

import { useState, useEffect } from 'react'
import { Loader2 } from 'lucide-react'
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
import { RegistryApiError } from '@/lib/types/registry'
import { logoutBrowserSession } from '@/lib/auth/session'
import type { ServerJSON } from '@/lib/types/registry'

interface InstallDialogProps {
  server: ServerJSON | null
  onClose: () => void
}

const GATEWAY_NAME_PATTERN = /^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$/
const BIDI_STRIP_RE = /[\u200B-\u200F\u202A-\u202E\u2060-\u2064\uFEFF]/g
const INVALID_CHARS_RE = /[^a-zA-Z0-9_-]/g

function deriveGatewayName(serverName: string): string {
  const segment = serverName.split('/').at(-1) ?? serverName
  return segment
    .normalize('NFC')
    .replace(BIDI_STRIP_RE, '')
    .replace(INVALID_CHARS_RE, '')
}

export function InstallDialog({ server, onClose }: InstallDialogProps) {
  const [gatewayName, setGatewayName] = useState('')
  const [bearerTokenEnv, setBearerTokenEnv] = useState('')
  const [nameError, setNameError] = useState<string | null>(null)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)

  // Re-derive gateway name whenever the server changes
  useEffect(() => {
    if (server) {
      setGatewayName(deriveGatewayName(server.name))
      setBearerTokenEnv('')
      setNameError(null)
      setSubmitError(null)
    }
  }, [server])

  const validateGatewayName = (value: string): string | null => {
    if (!value) return 'Gateway name is required'
    if (!GATEWAY_NAME_PATTERN.test(value))
      return 'Must start with a letter or digit and contain only letters, digits, underscores, or hyphens (max 64 chars)'
    return null
  }

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

    try {
      await installServer({
        name: server.name,
        gateway_name: gatewayName,
        version: server.version,
        bearer_token_env: bearerTokenEnv.trim() || undefined,
      })
      toast.success('Server installed successfully')
      onClose()
    } catch (error) {
      if (error instanceof RegistryApiError && error.status === 401) {
        await logoutBrowserSession()
        return
      }
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

          <div className="space-y-1.5">
            <Label htmlFor="bearer-token-env">Bearer token env var <span className="text-aurora-text-muted font-normal">(optional)</span></Label>
            <Input
              id="bearer-token-env"
              value={bearerTokenEnv}
              onChange={(e) => setBearerTokenEnv(e.target.value)}
              placeholder="MY_SERVER_TOKEN"
              disabled={isSubmitting}
            />
            <p className="text-xs text-aurora-text-muted">
              Name of the environment variable holding the bearer token for this server
            </p>
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
