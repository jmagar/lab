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
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
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
import { defaultGatewayBearerEnvName, validateBearerTokenEnvName } from '@/lib/gateway-env'
import { isAbortError } from '@/lib/api/service-action-client'

interface GatewayFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gateway: Gateway | null
  onSave: (input: CreateGatewayInput | UpdateGatewayInput) => Promise<void>
}

type FormMode = 'custom' | 'lab'
type GatewayAuthMode = 'none' | 'bearer'
type GatewayAuthSource = 'paste' | 'env'

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
  const abortControllerRef = useRef<AbortController | null>(null)
  const { data: supportedServices } = useSupportedServices()
  const { testGateway, saveServiceConfig, enableVirtualServer, disableVirtualServer } =
    useGatewayMutations()

  const [mode, setMode] = useState<FormMode>('lab')
  const [transport, setTransport] = useState<TransportType>('http')
  const [name, setName] = useState('')
  const [url, setUrl] = useState('')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')
  const [authMode, setAuthMode] = useState<GatewayAuthMode>('none')
  const [authSource, setAuthSource] = useState<GatewayAuthSource>('paste')
  const [bearerTokenEnv, setBearerTokenEnv] = useState('')
  const [bearerTokenValue, setBearerTokenValue] = useState('')
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
        setAuthMode(gateway.config.bearer_token_env ? 'bearer' : 'none')
        setAuthSource(gateway.config.bearer_token_env ? 'env' : 'paste')
        setBearerTokenEnv(gateway.config.bearer_token_env || '')
        setBearerTokenValue('')
        setProxyResources(gateway.config.proxy_resources ?? true)
      }
      } else {
        setMode('lab')
        setTransport(emptyCustomState.transport)
        setName(emptyCustomState.name)
        setUrl(emptyCustomState.url)
      setCommand(emptyCustomState.command)
      setArgs(emptyCustomState.args)
      setAuthMode('none')
      setAuthSource('paste')
      setBearerTokenEnv(emptyCustomState.bearerTokenEnv)
      setBearerTokenValue('')
      setProxyResources(emptyCustomState.proxyResources)
      setSelectedService('')
      setServiceValues({})
      setEnableServer(true)
    }
    setErrors({})
  }, [open, gateway])

  useEffect(() => {
    setServiceValues({})
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

    if (authMode === 'bearer') {
      if (authSource === 'env') {
        if (!bearerTokenEnv.trim()) {
          newErrors.bearerTokenEnv = 'Environment variable name is required'
        } else {
          const bearerTokenEnvError = validateBearerTokenEnvName(bearerTokenEnv)
          if (bearerTokenEnvError) {
            newErrors.bearerTokenEnv = bearerTokenEnvError
          }
        }
      } else {
        if (!bearerTokenValue.trim()) {
          newErrors.bearerTokenValue = 'Bearer token is required'
        }

        if (bearerTokenEnv.trim()) {
          const bearerTokenEnvError = validateBearerTokenEnvName(bearerTokenEnv)
          if (bearerTokenEnvError) {
            newErrors.bearerTokenEnv = bearerTokenEnvError
          }
        }
      }
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
      bearer_token_env:
        authMode === 'none'
          ? ''
          : authSource === 'env'
            ? bearerTokenEnv
            : bearerTokenEnv || undefined,
      bearer_token_value:
        authMode === 'bearer' && authSource === 'paste'
          ? bearerTokenValue
          : undefined,
      proxy_resources: proxyResources,
    },
  })

  const handleTest = async () => {
    if (isSaving) return
    if (!gateway || gateway.source === 'lab_service') {
      toast.info('Save and enable the gateway first, then test from the detail page.')
      return
    }

    if (!validateCustom()) return

    const controller = new AbortController()
    abortControllerRef.current = controller

    setIsTesting(true)
    try {
      const result = await testGateway(gateway.id)
      if (controller.signal.aborted) return
      if (result.severity === 'warning') {
        toast.warning(result.detail || result.message)
      } else if (result.success) {
        toast.success(`Connection successful: ${result.latency_ms}ms latency`)
      } else {
        toast.error(`Connection failed: ${result.error || result.message}`)
      }
    } catch (error) {
      if (isAbortError(error)) return
      toast.error(getErrorMessage(error, 'Failed to test connection'))
    } finally {
      setIsTesting(false)
    }
  }

  const handleSaveLab = async (): Promise<boolean> => {
    if (!validateLab() || !selectedService) return false

    const values = Object.fromEntries(
      Object.entries(serviceValues).filter(([field, value]) => {
        const configField = serviceConfig?.fields.find((item) => item.name === field)
        if (configField?.secret && configField.present && !value.trim()) {
          return false
        }
        return true
      }),
    )

    await saveServiceConfig(selectedService, values)
    if (enableServer) {
      await enableVirtualServer(selectedService)
    } else {
      await disableVirtualServer(selectedService)
    }
    return true
  }

  const handleSave = async () => {
    if (isTesting) return

    const controller = new AbortController()
    abortControllerRef.current = controller

    setIsSaving(true)
    try {
      if (mode === 'lab') {
        const saved = await handleSaveLab()
        if (controller.signal.aborted) return
        if (!saved) {
          return
        }
        toast.success(isEditing ? 'Lab gateway updated successfully' : 'Lab gateway configured successfully')
        onOpenChange(false)
        return
      }

      if (!validateCustom()) return
      await onSave(buildInput())
      if (controller.signal.aborted) return
      toast.success(isEditing ? 'Gateway updated successfully' : 'Gateway created successfully')
      onOpenChange(false)
    } catch (error) {
      if (isAbortError(error)) return
      toast.error(
        getErrorMessage(
          error,
          mode === 'lab'
            ? 'Failed to save Lab gateway'
            : isEditing
              ? 'Failed to update gateway'
              : 'Failed to create gateway',
        ),
      )
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={(nextOpen) => {
      if (!nextOpen) {
        abortControllerRef.current?.abort()
      }
      onOpenChange(nextOpen)
    }}>
        <DialogContent className="sm:max-w-[680px]">
        <DialogHeader>
          <DialogTitle>{isEditing ? 'Edit Gateway' : 'Add Gateway'}</DialogTitle>
          <DialogDescription>
            {mode === 'lab'
              ? 'Configure a Lab-backed gateway and decide whether to expose it in the control plane.'
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
                  Enable gateway
                </Label>
                <p className="text-sm text-muted-foreground">
                  Save canonical service config and expose this Lab service as a visible gateway.
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
              <Field className="space-y-4">
                <div className="space-y-1">
                  <FieldLabel>Authentication</FieldLabel>
                  <FieldDescription>
                    Choose how this gateway should authenticate upstream requests.
                  </FieldDescription>
                </div>

                <RadioGroup value={authMode} onValueChange={(value) => setAuthMode(value as GatewayAuthMode)}>
                  <label className="flex items-start gap-3 rounded-xl border p-4 cursor-pointer" htmlFor="auth-none">
                    <RadioGroupItem value="none" id="auth-none" />
                    <div className="space-y-1">
                      <span className="font-medium text-sm">No auth</span>
                      <p className="text-sm text-muted-foreground">
                        Use this when the upstream does not require an Authorization header.
                      </p>
                    </div>
                  </label>
                  <label className="flex items-start gap-3 rounded-xl border p-4 cursor-pointer" htmlFor="auth-bearer">
                    <RadioGroupItem value="bearer" id="auth-bearer" />
                    <div className="space-y-1">
                      <span className="font-medium text-sm">Bearer token</span>
                      <p className="text-sm text-muted-foreground">
                        Recommended for GitHub and other remote HTTP MCP servers.
                      </p>
                    </div>
                  </label>
                </RadioGroup>

                {authMode === 'bearer' && (
                  <div className="space-y-4 rounded-xl border p-4">
                    <RadioGroup value={authSource} onValueChange={(value) => setAuthSource(value as GatewayAuthSource)}>
                      <label className="flex items-start gap-3 rounded-lg border p-3 cursor-pointer" htmlFor="auth-source-paste">
                        <RadioGroupItem value="paste" id="auth-source-paste" />
                        <div className="space-y-1">
                          <span className="font-medium text-sm">Paste token</span>
                          <p className="text-sm text-muted-foreground">
                            Paste the secret here and Labby will store it in <code>~/.lab/.env</code> for you.
                          </p>
                        </div>
                      </label>
                      <label className="flex items-start gap-3 rounded-lg border p-3 cursor-pointer" htmlFor="auth-source-env">
                        <RadioGroupItem value="env" id="auth-source-env" />
                        <div className="space-y-1">
                          <span className="font-medium text-sm">Use existing env var</span>
                          <p className="text-sm text-muted-foreground">
                            Reference an existing environment variable instead of entering a secret here.
                          </p>
                        </div>
                      </label>
                    </RadioGroup>

                    {authSource === 'paste' ? (
                      <FieldGroup>
                        <Field>
                          <FieldLabel htmlFor="bearer-token-value">Bearer token</FieldLabel>
                          <Input
                            id="bearer-token-value"
                            type="password"
                            value={bearerTokenValue}
                            onChange={(event) => setBearerTokenValue(event.target.value)}
                            placeholder="ghp_..."
                            className={errors.bearerTokenValue ? 'border-destructive' : ''}
                          />
                          {errors.bearerTokenValue ? (
                            <p className="text-sm text-destructive">{errors.bearerTokenValue}</p>
                          ) : (
                            <FieldDescription>
                              Paste the token only. Labby will add the <code>Bearer</code> prefix automatically if needed.
                            </FieldDescription>
                          )}
                        </Field>
                        <Field>
                          <FieldLabel htmlFor="bearer-token-env-override">Env var name</FieldLabel>
                          <Input
                            id="bearer-token-env-override"
                            value={bearerTokenEnv}
                            onChange={(event) => setBearerTokenEnv(event.target.value)}
                            placeholder={defaultGatewayBearerEnvName(name || 'gateway')}
                            className={errors.bearerTokenEnv ? 'border-destructive' : ''}
                          />
                          {errors.bearerTokenEnv ? (
                            <p className="text-sm text-destructive">{errors.bearerTokenEnv}</p>
                          ) : (
                            <FieldDescription>
                              Optional. Leave blank to let Labby generate an env var name automatically.
                            </FieldDescription>
                          )}
                        </Field>
                      </FieldGroup>
                    ) : (
                      <Field>
                        <FieldLabel htmlFor="bearer-token-env">Bearer token env var</FieldLabel>
                        <Input
                          id="bearer-token-env"
                          value={bearerTokenEnv}
                          onChange={(event) => setBearerTokenEnv(event.target.value)}
                          placeholder={defaultGatewayBearerEnvName(name || 'gateway')}
                          className={errors.bearerTokenEnv ? 'border-destructive' : ''}
                        />
                        {errors.bearerTokenEnv ? (
                          <p className="text-sm text-destructive">{errors.bearerTokenEnv}</p>
                        ) : (
                          <FieldDescription>
                            Enter the env var name only. The env var value can be a bare token or a full <code>Bearer ...</code> header.
                          </FieldDescription>
                        )}
                      </Field>
                    )}
                  </div>
                )}
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
              disabled={isTesting || isSaving}
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
          <Button onClick={handleSave} disabled={isSaving || isTesting}>
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
