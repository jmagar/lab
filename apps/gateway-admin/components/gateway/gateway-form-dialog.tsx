'use client'

import { useEffect, useMemo, useRef, useState } from 'react'
import { Loader2, Play, ShieldCheck, AlertCircle, CheckCircle2, ChevronRight } from 'lucide-react'
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
import { GatewayApiError } from '@/lib/api/gateway-client-core'
import { upstreamOauthApi } from '@/lib/api/upstream-oauth-client'
import { useUpstreamOauthStatus } from '@/lib/hooks/use-upstream-oauth'
import type { OAuthConnectState } from '@/lib/types/upstream-oauth'
import { Badge } from '@/components/ui/badge'

interface GatewayFormDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gateway: Gateway | null
  onSave: (input: CreateGatewayInput | UpdateGatewayInput) => Promise<void>
}

type FormMode = 'custom' | 'lab'
type GatewayAuthMode = 'none' | 'bearer' | 'oauth'
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
  const probeInfoRef = useRef<{ registration_strategy: string; scopes?: string[] } | null>(null)
  const nameAutoRef = useRef(false)
  const { data: supportedServices } = useSupportedServices()
  const { testGateway, saveServiceConfig, enableVirtualServer, disableVirtualServer } =
    useGatewayMutations()

  const [mode, setMode] = useState<FormMode>('custom')
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
  const [saveError, setSaveError] = useState<string | null>(null)
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [oauthState, setOauthState] = useState<OAuthConnectState>({ kind: 'idle' })
  const [oauthProbed, setOauthProbed] = useState<{ oauth_discovered: boolean; upstream: string; issuer?: string; scopes?: string[]; registration_strategy?: string } | null>(null)
  const [isProbing, setIsProbing] = useState(false)

  const serviceMeta = useMemo(
    () => supportedServices?.find((service) => service.key === selectedService) ?? null,
    [selectedService, supportedServices],
  )
  const serviceEnvFields = useMemo(() => serviceFields(serviceMeta), [serviceMeta])
  const { data: serviceConfig } = useServiceConfig(mode === 'lab' && selectedService ? selectedService : null)

  const oauthUpstream = oauthState.kind === 'authorizing' || oauthState.kind === 'connected' || oauthState.kind === 'discovered'
    ? (oauthState as { upstream: string }).upstream
    : null
  const { data: oauthStatus } = useUpstreamOauthStatus(
    oauthState.kind === 'authorizing' ? oauthUpstream : null,
    { pollWhilePending: oauthState.kind === 'authorizing' },
  )

  useEffect(() => {
    if (oauthState.kind === 'authorizing' && oauthStatus?.authenticated) {
      const info = probeInfoRef.current
      setOauthState({
        kind: 'connected',
        upstream: oauthState.upstream,
        registration_strategy: info?.registration_strategy ?? 'dynamic',
        scopes: info?.scopes,
      })
    }
  }, [oauthState, oauthStatus?.authenticated])

  // Auto-probe the URL for OAuth support when transport is HTTP and URL looks valid.
  // Resets probed state and authMode when URL changes so stale OAuth option disappears.
  useEffect(() => {
    if (transport !== 'http' || !url.trim()) {
      setOauthProbed(null)
      setIsProbing(false)
      if (authMode === 'oauth') setAuthMode('none')
      return
    }
    setOauthProbed(null)
    let cancelled = false
    const timer = setTimeout(() => {
      setIsProbing(true)
      upstreamOauthApi.probe(url.trim()).then((result) => {
        if (!cancelled) { setOauthProbed(result); setIsProbing(false) }
      }).catch(() => {
        if (!cancelled) { setOauthProbed({ oauth_discovered: false, upstream: '' }); setIsProbing(false) }
      })
    }, 600)
    return () => {
      cancelled = true
      setIsProbing(false)
      clearTimeout(timer)
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url, transport])

  // Auto-fill the name from the URL hostname when the user hasn't typed a name yet.
  useEffect(() => {
    if (isEditing || transport !== 'http' || !url.trim()) return
    try {
      const hostname = new URL(url).hostname.replace(/^www\./, '')
      const slug = hostname.replace(/[^a-z0-9]+/gi, '-').toLowerCase().replace(/^-+|-+$/g, '')
      if (!slug) return
      setName((prev) => {
        if (!prev || nameAutoRef.current) {
          nameAutoRef.current = true
          return slug
        }
        return prev
      })
    } catch {
      // invalid URL, skip
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url])

  async function handleOauthConnect() {
    if (!url.trim()) return
    // Open a blank tab synchronously — must happen directly in the click handler
    // before any await, otherwise browsers treat it as an unsolicited popup and block it.
    const authTab = window.open('about:blank', '_blank')
    setOauthState({ kind: 'probing' })
    try {
      // Reuse already-probed result to avoid a duplicate round-trip.
      const probe = oauthProbed?.oauth_discovered ? oauthProbed : await upstreamOauthApi.probe(url.trim())
      if (!probe.oauth_discovered) {
        authTab?.close()
        setOauthState({ kind: 'error', message: 'This server does not advertise OAuth support' })
        return
      }
      setOauthState({ kind: 'discovered', upstream: probe.upstream, issuer: probe.issuer, scopes: probe.scopes })
      probeInfoRef.current = { registration_strategy: probe.registration_strategy ?? 'dynamic', scopes: probe.scopes }
      const { authorization_url } = await upstreamOauthApi.start(probe.upstream)
      if (!authTab || authTab.closed) {
        setOauthState({ kind: 'error', message: 'Authorization tab was closed. Please try again.' })
        return
      }
      authTab.location.href = authorization_url
      setOauthState({ kind: 'authorizing', upstream: probe.upstream })
    } catch (err: unknown) {
      authTab?.close()
      setOauthState({ kind: 'error', message: err instanceof Error ? err.message : 'OAuth connection failed' })
    }
  }

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
        const initialAuthMode = gateway.config.oauth_enabled ? 'oauth'
          : gateway.config.bearer_token_env ? 'bearer'
          : 'none'
        setAuthMode(initialAuthMode)
        if (initialAuthMode === 'oauth') {
          setOauthState({ kind: 'connected', upstream: gateway.name, registration_strategy: 'unknown', scopes: undefined })
          setOauthProbed({ oauth_discovered: true, upstream: gateway.name })
        }
        setAuthSource(gateway.config.bearer_token_env ? 'env' : 'paste')
        setBearerTokenEnv(gateway.config.bearer_token_env || '')
        setBearerTokenValue('')
        setProxyResources(gateway.config.proxy_resources ?? true)
      }
      } else {
        setMode('custom')
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
        nameAutoRef.current = false
      }
    setErrors({})
    setOauthState({ kind: 'idle' })
    setOauthProbed(null)
  }, [open, gateway])

  useEffect(() => {
    setServiceValues({})
  }, [selectedService])

  useEffect(() => {
    setOauthState({ kind: 'idle' })
  }, [url])

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

    if (authMode === 'oauth') {
      if (oauthState.kind !== 'connected' && !oauthStatus?.authenticated) {
        newErrors.oauth = 'Complete OAuth authorization before saving'
      }
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
        authMode === 'none' || authMode === 'oauth'
          ? ''
          : authSource === 'env'
            ? bearerTokenEnv
            : bearerTokenEnv || undefined,
      bearer_token_value:
        authMode === 'bearer' && authSource === 'paste'
          ? bearerTokenValue
          : undefined,
      oauth:
        authMode === 'oauth' && oauthState.kind === 'connected' && oauthState.registration_strategy !== 'unknown'
          ? { registration_strategy: oauthState.registration_strategy, scopes: oauthState.scopes }
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
      setSaveError(null)
      await onSave(buildInput())
      if (controller.signal.aborted) return
      toast.success(isEditing ? 'Gateway updated successfully' : 'Gateway created successfully')
      onOpenChange(false)
    } catch (error) {
      if (isAbortError(error)) return
      if (error instanceof GatewayApiError && error.status === 409) {
        setSaveError(error.message)
        return
      }
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
        <DialogHeader className="shrink-0">
          <DialogTitle>{isEditing ? 'Edit Gateway' : 'Add Gateway'}</DialogTitle>
          <DialogDescription>
            {isEditing
              ? 'Edit gateway settings.'
              : mode === 'lab'
                ? 'Connect a built-in Lab service.'
                : 'Connect an upstream MCP server.'}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 min-h-0 overflow-y-auto -mx-6 px-6">
        <Tabs
          value={mode}
          onValueChange={(value) => setMode(value as FormMode)}
          className="space-y-6"
        >
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="lab" disabled={isEditing && !isLabGateway}>
              Lab Service
            </TabsTrigger>
            <TabsTrigger value="custom" disabled={isEditing && isLabGateway}>
              Custom
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
                  onChange={(event) => { nameAutoRef.current = false; setName(event.target.value) }}
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

            <RadioGroup
              value={transport}
              onValueChange={(value) => setTransport(value as TransportType)}
              className="grid grid-cols-2 gap-3"
            >
              <label className="flex items-start gap-3 rounded-xl border p-4 cursor-pointer" htmlFor="transport-http">
                <RadioGroupItem value="http" id="transport-http" />
                <div className="space-y-0.5">
                  <span className="font-medium text-sm">HTTP</span>
                  <p className="text-sm text-muted-foreground">Remote server via HTTP or SSE</p>
                </div>
              </label>
              <label className="flex items-start gap-3 rounded-xl border p-4 cursor-pointer" htmlFor="transport-stdio">
                <RadioGroupItem value="stdio" id="transport-stdio" />
                <div className="space-y-0.5">
                  <span className="font-medium text-sm">stdio</span>
                  <p className="text-sm text-muted-foreground">Local process via stdin/stdout</p>
                </div>
              </label>
            </RadioGroup>

            {transport === 'http' && (
              <FieldGroup>
                <Field>
                  <FieldLabel htmlFor="url">URL</FieldLabel>
                  <div className="relative">
                    <Input
                      id="url"
                      value={url}
                      onChange={(event) => setUrl(event.target.value)}
                      placeholder="http://localhost:3001/mcp"
                      className={`${errors.url ? 'border-destructive' : ''} pr-8`}
                    />
                    {isProbing && (
                      <Loader2 className="absolute right-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground animate-spin pointer-events-none" />
                    )}
                    {!isProbing && oauthProbed?.oauth_discovered && (
                      <CheckCircle2 className="absolute right-2.5 top-1/2 -translate-y-1/2 size-4 text-green-500 pointer-events-none" />
                    )}
                  </div>
                  {errors.url && <p className="text-sm text-destructive">{errors.url}</p>}
                </Field>
              </FieldGroup>
            )}

            {transport === 'stdio' && (
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
            )}

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
                      <p className="text-sm text-muted-foreground">No Authorization header sent.</p>
                    </div>
                  </label>
                  <label className="flex items-start gap-3 rounded-xl border p-4 cursor-pointer" htmlFor="auth-bearer">
                    <RadioGroupItem value="bearer" id="auth-bearer" />
                    <div className="space-y-1">
                      <span className="font-medium text-sm">Bearer token</span>
                      <p className="text-sm text-muted-foreground">Static token sent as an Authorization header.</p>
                    </div>
                  </label>
                  {transport === 'http' && (oauthProbed?.oauth_discovered || gateway?.config.oauth_enabled) && (
                    <label className="flex items-start gap-3 rounded-xl border p-4 cursor-pointer" htmlFor="auth-oauth">
                      <RadioGroupItem value="oauth" id="auth-oauth" />
                      <div className="space-y-1">
                        <span className="font-medium text-sm">
                          OAuth (MCP)
                          {oauthProbed?.oauth_discovered && (
                            <Badge variant="secondary" className="ml-2 text-xs">Detected</Badge>
                          )}
                        </span>
                        <p className="text-sm text-muted-foreground">OAuth 2.1 — for GitHub, Cloudflare, and other remote MCP servers.</p>
                      </div>
                    </label>
                  )}
                </RadioGroup>

                {authMode === 'oauth' && (
                  <div className="rounded-lg border p-4 flex flex-col gap-3">
                    {oauthState.kind === 'connected' ? (
                      <div className="flex items-center justify-between gap-2">
                        <div className="flex items-center gap-2 text-sm text-green-700 dark:text-green-400 font-medium">
                          <ShieldCheck className="size-4" />
                          Connected
                          <Badge variant="outline" className="border-green-500 text-green-600 ml-1">Authorized</Badge>
                        </div>
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => {
                            setOauthState({ kind: 'idle' })
                            probeInfoRef.current = null
                          }}
                        >
                          Re-authorize
                        </Button>
                      </div>
                    ) : (
                      <>
                        <p className="text-sm text-muted-foreground">
                          {!url.trim()
                            ? 'Enter a URL above, then connect.'
                            : oauthState.kind === 'authorizing'
                              ? 'Complete authorization in the new tab…'
                              : 'Connect this gateway via OAuth. A popup will open for you to authorize.'}
                        </p>
                        {oauthState.kind === 'error' && (
                          <div className="flex items-start gap-2 text-sm text-destructive">
                            <AlertCircle className="size-4 mt-0.5 shrink-0" />
                            {oauthState.message}
                          </div>
                        )}
                        <Button
                          type="button"
                          size="sm"
                          onClick={() => void handleOauthConnect()}
                          disabled={!url.trim() || oauthState.kind === 'probing' || oauthState.kind === 'authorizing'}
                        >
                          {(oauthState.kind === 'probing' || oauthState.kind === 'authorizing') && (
                            <Loader2 className="size-4 mr-2 animate-spin" />
                          )}
                          {oauthState.kind === 'probing' ? 'Detecting OAuth…' :
                           oauthState.kind === 'authorizing' ? 'Waiting…' : 'Connect via OAuth'}
                        </Button>
                      </>
                    )}
                  </div>
                )}
                {errors.oauth && <p className="text-sm text-destructive">{errors.oauth}</p>}

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
                            autoComplete="new-password"
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
                        <details className="group">
                          <summary className="flex cursor-pointer select-none list-none items-center gap-1 text-sm text-muted-foreground [&::-webkit-details-marker]:hidden">
                            <ChevronRight className="size-3 transition-transform group-open:rotate-90" />
                            Advanced
                          </summary>
                          <div className="mt-3">
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
                          </div>
                        </details>
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
        </div>

        {saveError && (
          <div className="shrink-0 flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive">
            <AlertCircle className="size-4 mt-0.5 shrink-0" />
            <span>{saveError}</span>
          </div>
        )}
        <DialogFooter className="shrink-0 gap-2 sm:gap-0">
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
