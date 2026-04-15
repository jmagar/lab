'use client'

import { useEffect, useMemo, useRef, useState } from 'react'
import { Loader2, Play } from 'lucide-react'
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
import { useGatewayMutations, useServiceConfig, useSupportedServices } from '@/lib/hooks/use-gateways'
import { LabServicePicker } from './lab-service-picker'
import type {
  Gateway,
  CreateGatewayInput,
  UpdateGatewayInput,
  TransportType,
  SupportedService,
} from '@/lib/types/gateway'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/utils'

interface GatewayFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gateway: Gateway | null
  onSave: (input: CreateGatewayInput | UpdateGatewayInput) => Promise<void>
}

type FormMode = 'custom' | 'lab'

function valuePreview(fieldName: string, preview?: string | null) {
  return preview ?? (fieldName.endsWith('_URL') ? 'http://localhost' : '')
}

const emptyCustomState = {
  transport: 'http' as TransportType,
  name: '',
  url: '',
  command: '',
  args: '',
  bearerTokenEnv: '',
  proxyResources: true,
}

function serviceFields(serviceMeta: SupportedService | null) {
  return serviceMeta ? [...serviceMeta.required_env, ...serviceMeta.optional_env] : []
}

export function GatewayFormDialog({
  open,
  onOpenChange,
  gateway,
  onSave,
}: GatewayFormDialogProps) {
  const isEditing = !!gateway
  const isLabGateway = gateway?.source === 'lab_service'
  const prevOpenRef = useRef(false)
  const { data: supportedServices } = useSupportedServices()
  const { testGateway, saveServiceConfig, enableVirtualServer, disableVirtualServer } =
    useGatewayMutations()

  const [mode, setMode] = useState<FormMode>('lab')
  const [transport, setTransport] = useState<TransportType>('http')
  const [name, setName] = useState('')
  const [url, setUrl] = useState('')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')
  const [bearerTokenEnv, setBearerTokenEnv] = useState('')
  const [proxyResources, setProxyResources] = useState(true)

  const [selectedService, setSelectedService] = useState('')
  const [serviceValues, setServiceValues] = useState<Record<string, string>>({})
  const [enableServer, setEnableServer] = useState(true)

  const [isSaving, setIsSaving] = useState(false)
  const [isTesting, setIsTesting] = useState(false)
  const [errors, setErrors] = useState<Record<string, string>>({})

  const serviceMeta = useMemo(
    () => supportedServices?.find((service) => service.key === selectedService) ?? null,
    [selectedService, supportedServices],
  )
  const serviceEnvFields = useMemo(() => serviceFields(serviceMeta), [serviceMeta])
  const { data: serviceConfig } = useServiceConfig(mode === 'lab' && selectedService ? selectedService : null)

  useEffect(() => {
    const wasOpen = prevOpenRef.current
    prevOpenRef.current = open
    if (!open || wasOpen) return

    if (gateway) {
      if (gateway.source === 'lab_service') {
        setMode('lab')
        setSelectedService(gateway.id)
        setEnableServer(gateway.enabled ?? true)
      } else {
        setMode('custom')
        setTransport(gateway.transport === 'lab_service' ? 'http' : gateway.transport)
        setName(gateway.name)
        setUrl(gateway.config.url || '')
        setCommand(gateway.config.command || '')
        setArgs(gateway.config.args?.join(' ') || '')
        setBearerTokenEnv(gateway.config.bearer_token_env || '')
        setProxyResources(gateway.config.proxy_resources ?? true)
      }
      } else {
        setMode('lab')
        setTransport(emptyCustomState.transport)
        setName(emptyCustomState.name)
        setUrl(emptyCustomState.url)
      setCommand(emptyCustomState.command)
      setArgs(emptyCustomState.args)
      setBearerTokenEnv(emptyCustomState.bearerTokenEnv)
      setProxyResources(emptyCustomState.proxyResources)
      setSelectedService('')
      setServiceValues({})
      setEnableServer(true)
    }
    setErrors({})
  }, [open, gateway])

  useEffect(() => {
    if (!selectedService) {
      setServiceValues({})
    }
  }, [selectedService])

  useEffect(() => {
    if (!serviceMeta || !serviceConfig) return

    const nextValues: Record<string, string> = {}
    for (const field of serviceEnvFields) {
      const configField = serviceConfig.fields.find((item) => item.name === field.name)
      nextValues[field.name] = valuePreview(field.name, configField?.value_preview)
    }
    setServiceValues(nextValues)
  }, [serviceConfig, serviceEnvFields])

  const validateCustom = () => {
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
    } else if (!command.trim()) {
      newErrors.command = 'Command is required'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const validateLab = () => {
    const newErrors: Record<string, string> = {}
    if (!selectedService) {
      newErrors.service = 'Choose a Lab service'
    }
    for (const field of serviceMeta?.required_env ?? []) {
      const configField = serviceConfig?.fields.find((item) => item.name === field.name)
      const keepExistingSecret = field.secret && configField?.present && !serviceValues[field.name]?.trim()
      if (!keepExistingSecret && !serviceValues[field.name]?.trim()) {
        newErrors[field.name] = `${field.name} is required`
      }
    }
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const buildInput = (): CreateGatewayInput => ({
    name,
    transport,
    config: {
      ...(transport === 'http'
        ? { url }
        : {
            command,
            args: args.trim() ? args.split(/\s+/) : undefined,
          }),
      bearer_token_env: bearerTokenEnv || undefined,
      proxy_resources: proxyResources,
    },
  })

  const handleTest = async () => {
    if (!gateway || gateway.source === 'lab_service') {
      toast.info('Save and enable the server first, then test from the detail page.')
      return
    }

    if (!validateCustom()) return

    setIsTesting(true)
    try {
      const result = await testGateway(gateway.id)
      if (result.severity === 'warning') {
        toast.warning(result.detail || result.message)
      } else if (result.success) {
        toast.success(`Connection successful: ${result.latency_ms}ms latency`)
      } else {
        toast.error(`Connection failed: ${result.error || result.message}`)
      }
    } catch (error) {
      toast.error(getErrorMessage(error, 'Failed to test connection'))
    } finally {
      setIsTesting(false)
    }
  }

  const handleSaveLab = async () => {
    if (!validateLab() || !selectedService) return

    const values = Object.fromEntries(
      Object.entries(serviceValues).filter(([field, value]) => {
        const configField = serviceConfig?.fields.find((item) => item.name === field)
        if (!value.trim()) {
          return false
        }
        return !(configField?.secret && configField.present && configField.value_preview == null)
      }),
    )

    await saveServiceConfig(selectedService, values)
    if (enableServer) {
      await enableVirtualServer(selectedService)
    } else {
      await disableVirtualServer(selectedService)
    }
  }

  const handleSave = async () => {
    setIsSaving(true)
    try {
      if (mode === 'lab') {
        await handleSaveLab()
        toast.success(isEditing ? 'Lab gateway updated successfully' : 'Lab gateway configured successfully')
        onOpenChange(false)
        return
      }

      if (!validateCustom()) return
      await onSave(buildInput())
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[680px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? 'Edit Server' : 'Add Server'}</DialogTitle>
          <DialogDescription>
            {mode === 'lab'
              ? 'Configure a Lab-backed virtual server and decide whether to expose it in the gateway.'
              : 'Configure a custom upstream MCP server connection.'}
          </DialogDescription>
        </DialogHeader>

        <Tabs
          value={mode}
          onValueChange={(value) => setMode(value as FormMode)}
          className="space-y-6"
        >
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="lab" disabled={isEditing && !isLabGateway}>
              Lab Gateways
            </TabsTrigger>
            <TabsTrigger value="custom" disabled={isEditing && isLabGateway}>
              Custom Gateways
            </TabsTrigger>
          </TabsList>

          <TabsContent value="lab" className="space-y-6">
            <FieldGroup>
              <Field>
                <LabServicePicker
                  selectedService={selectedService}
                  services={supportedServices ?? []}
                  onSelect={setSelectedService}
                />
                {errors.service && <p className="text-sm text-destructive">{errors.service}</p>}
              </Field>
            </FieldGroup>

            {serviceMeta && (
              <FieldGroup>
                {serviceEnvFields.map((field) => {
                  const configField = serviceConfig?.fields.find((item) => item.name === field.name)
                  const hasStoredSecret = field.secret && configField?.present

                  return (
                    <Field key={field.name}>
                    <FieldLabel htmlFor={field.name}>{field.name}</FieldLabel>
                    <Input
                      id={field.name}
                      type={field.secret ? 'password' : 'text'}
                      value={serviceValues[field.name] ?? ''}
                      onChange={(event) =>
                        setServiceValues((current) => ({
                          ...current,
                          [field.name]: event.target.value,
                        }))
                      }
                      placeholder={hasStoredSecret ? 'Leave blank to keep current value' : field.example}
                      className={errors[field.name] ? 'border-destructive' : ''}
                    />
                    {errors[field.name] ? (
                      <p className="text-sm text-destructive">{errors[field.name]}</p>
                    ) : (
                      <FieldDescription>
                        {field.description}
                        {hasStoredSecret ? ' Current secret is already configured.' : ''}
                      </FieldDescription>
                    )}
                    </Field>
                  )
                })}
              </FieldGroup>
            )}

            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-0.5">
                <Label htmlFor="enable-virtual-server" className="font-medium">
                  Enable as gateway server
                </Label>
                <p className="text-sm text-muted-foreground">
                  Save canonical service config and expose this service as a visible virtual server.
                </p>
              </div>
              <Switch
                id="enable-virtual-server"
                checked={enableServer}
                onCheckedChange={setEnableServer}
              />
            </div>
          </TabsContent>

          <TabsContent value="custom" className="space-y-6">
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="name">Name</FieldLabel>
                <Input
                  id="name"
                  value={name}
                  onChange={(event) => setName(event.target.value)}
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

            <Tabs value={transport} onValueChange={(value) => setTransport(value as TransportType)}>
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
                      onChange={(event) => setUrl(event.target.value)}
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
                      onChange={(event) => setCommand(event.target.value)}
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
                      onChange={(event) => setArgs(event.target.value)}
                      placeholder="-y @modelcontextprotocol/server-filesystem /path"
                    />
                    <FieldDescription>Space-separated command arguments</FieldDescription>
                  </Field>
                </FieldGroup>
              </TabsContent>
            </Tabs>

            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="bearer-token">Bearer Token Env</FieldLabel>
                <Input
                  id="bearer-token"
                  value={bearerTokenEnv}
                  onChange={(event) => setBearerTokenEnv(event.target.value)}
                  placeholder="MY_API_TOKEN"
                />
                <FieldDescription>Environment variable containing the auth token</FieldDescription>
              </Field>
            </FieldGroup>

            <div className="flex items-center justify-between rounded-lg border p-4">
              <div className="space-y-0.5">
                <Label htmlFor="proxy-resources" className="font-medium">
                  Proxy Resources
                </Label>
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
          </TabsContent>
        </Tabs>

        <DialogFooter className="gap-2 sm:gap-0">
          {isEditing && !isLabGateway && (
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
            {mode === 'lab'
              ? isEditing
                ? 'Save Service'
                : 'Configure Service'
              : isEditing
                ? 'Save Changes'
                : 'Add Gateway'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
