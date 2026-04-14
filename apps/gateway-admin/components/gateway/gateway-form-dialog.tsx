'use client'

import { useState, useEffect } from 'react'
import { Play, Loader2 } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { FieldGroup, Field, FieldLabel, FieldDescription } from '@/components/ui/field'
import type { Gateway, CreateGatewayInput, UpdateGatewayInput, TransportType } from '@/lib/types/gateway'
import { useGatewayMutations } from '@/lib/hooks/use-gateways'
import { toast } from 'sonner'

interface GatewayFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gateway: Gateway | null
  onSave: (input: CreateGatewayInput | UpdateGatewayInput) => Promise<void>
}

export function GatewayFormDialog({ 
  open, 
  onOpenChange, 
  gateway, 
  onSave 
}: GatewayFormDialogProps) {
  const isEditing = !!gateway
  const { testGateway } = useGatewayMutations()

  const [transport, setTransport] = useState<TransportType>('http')
  const [name, setName] = useState('')
  const [url, setUrl] = useState('')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')
  const [bearerTokenEnv, setBearerTokenEnv] = useState('')
  const [proxyResources, setProxyResources] = useState(true)
  
  const [isSaving, setIsSaving] = useState(false)
  const [isTesting, setIsTesting] = useState(false)
  const [errors, setErrors] = useState<Record<string, string>>({})

  // Reset form when dialog opens
  useEffect(() => {
    if (open) {
      if (gateway) {
        setTransport(gateway.transport)
        setName(gateway.name)
        setUrl(gateway.config.url || '')
        setCommand(gateway.config.command || '')
        setArgs(gateway.config.args?.join(' ') || '')
        setBearerTokenEnv(gateway.config.bearer_token_env || '')
        setProxyResources(gateway.config.proxy_resources ?? true)
      } else {
        setTransport('http')
        setName('')
        setUrl('')
        setCommand('')
        setArgs('')
        setBearerTokenEnv('')
        setProxyResources(true)
      }
      setErrors({})
    }
  }, [open, gateway])

  const validate = () => {
    const newErrors: Record<string, string> = {}
    
    if (!name.trim()) {
      newErrors.name = 'Name is required'
    } else if (!/^[a-z0-9-]+$/.test(name)) {
      newErrors.name = 'Name must be lowercase alphanumeric with hyphens'
    }

    if (transport === 'http') {
      if (!url.trim()) {
        newErrors.url = 'URL is required'
      } else {
        try {
          new URL(url)
        } catch {
          newErrors.url = 'Invalid URL format'
        }
      }
    } else {
      if (!command.trim()) {
        newErrors.command = 'Command is required'
      }
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const buildInput = (): CreateGatewayInput => ({
    name,
    transport,
    config: {
      ...(transport === 'http' ? { url } : { 
        command, 
        args: args.trim() ? args.split(/\s+/) : undefined 
      }),
      bearer_token_env: bearerTokenEnv || undefined,
      proxy_resources: proxyResources,
    },
  })

  const handleTest = async () => {
    if (!validate()) return
    
    setIsTesting(true)
    try {
      // For new gateways, we can't actually test yet
      if (!gateway) {
        toast.info('Save the gateway first to test the connection')
        return
      }
      
      const result = await testGateway(gateway.id)
      if (result.success) {
        toast.success(`Connection successful: ${result.latency_ms}ms latency`)
      } else {
        toast.error(`Connection failed: ${result.error || result.message}`)
      }
    } catch {
      toast.error('Failed to test connection')
    } finally {
      setIsTesting(false)
    }
  }

  const handleSave = async () => {
    if (!validate()) return

    setIsSaving(true)
    try {
      await onSave(buildInput())
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? 'Edit Gateway' : 'Add Gateway'}</DialogTitle>
          <DialogDescription>
            {isEditing 
              ? 'Update the gateway configuration settings.'
              : 'Configure a new upstream MCP server connection.'
            }
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Name field */}
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="name">Name</FieldLabel>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="my-gateway"
                className={errors.name ? 'border-destructive' : ''}
              />
              {errors.name ? (
                <p className="text-sm text-destructive">{errors.name}</p>
              ) : (
                <FieldDescription>Lowercase alphanumeric with hyphens</FieldDescription>
              )}
            </Field>
          </FieldGroup>

          {/* Transport selection */}
          <Tabs value={transport} onValueChange={(v) => setTransport(v as TransportType)}>
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="http">HTTP</TabsTrigger>
              <TabsTrigger value="stdio">stdio</TabsTrigger>
            </TabsList>
            
            <TabsContent value="http" className="space-y-4 mt-4">
              <FieldGroup>
                <Field>
                  <FieldLabel htmlFor="url">URL</FieldLabel>
                  <Input
                    id="url"
                    value={url}
                    onChange={(e) => setUrl(e.target.value)}
                    placeholder="http://localhost:3001/mcp"
                    className={errors.url ? 'border-destructive' : ''}
                  />
                  {errors.url && <p className="text-sm text-destructive">{errors.url}</p>}
                </Field>
              </FieldGroup>
            </TabsContent>

            <TabsContent value="stdio" className="space-y-4 mt-4">
              <FieldGroup>
                <Field>
                  <FieldLabel htmlFor="command">Command</FieldLabel>
                  <Input
                    id="command"
                    value={command}
                    onChange={(e) => setCommand(e.target.value)}
                    placeholder="npx"
                    className={errors.command ? 'border-destructive' : ''}
                  />
                  {errors.command && <p className="text-sm text-destructive">{errors.command}</p>}
                </Field>
                <Field>
                  <FieldLabel htmlFor="args">Arguments</FieldLabel>
                  <Input
                    id="args"
                    value={args}
                    onChange={(e) => setArgs(e.target.value)}
                    placeholder="-y @modelcontextprotocol/server-filesystem /path"
                  />
                  <FieldDescription>Space-separated command arguments</FieldDescription>
                </Field>
              </FieldGroup>
            </TabsContent>
          </Tabs>

          {/* Additional options */}
          <FieldGroup>
            <Field>
              <FieldLabel htmlFor="bearer-token">Bearer Token Env</FieldLabel>
              <Input
                id="bearer-token"
                value={bearerTokenEnv}
                onChange={(e) => setBearerTokenEnv(e.target.value)}
                placeholder="MY_API_TOKEN"
              />
              <FieldDescription>Environment variable containing the auth token</FieldDescription>
            </Field>
          </FieldGroup>

          <div className="flex items-center justify-between rounded-lg border p-4">
            <div className="space-y-0.5">
              <Label htmlFor="proxy-resources" className="font-medium">Proxy Resources</Label>
              <p className="text-sm text-muted-foreground">
                Forward MCP resource requests to this gateway
              </p>
            </div>
            <Switch
              id="proxy-resources"
              checked={proxyResources}
              onCheckedChange={setProxyResources}
            />
          </div>
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          {isEditing && (
            <Button
              type="button"
              variant="outline"
              onClick={handleTest}
              disabled={isTesting}
              className="mr-auto"
            >
              {isTesting ? (
                <Loader2 className="size-4 mr-2 animate-spin" />
              ) : (
                <Play className="size-4 mr-2" />
              )}
              Test
            </Button>
          )}
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={isSaving}>
            {isSaving && <Loader2 className="size-4 mr-2 animate-spin" />}
            {isEditing ? 'Save Changes' : 'Add Gateway'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
