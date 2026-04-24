'use client'

import { useEffect, useState } from 'react'
import { formatDistanceToNow } from 'date-fns'
import * as React from 'react'
import {
  Check,
  CheckCircle2,
  ChevronDown,
  ChevronRight,
  Copy,
  ExternalLink,
  FileCode,
  Globe,
  Package,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Spinner } from '@/components/ui/spinner'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { safeHref } from '@/lib/utils/safe-href'
import { githubAvatarFromRepoUrl } from '@/lib/github-avatar'
import {
  formatLocalDateTimePrecise,
  formatUtcTooltip,
} from '@/lib/utils/format-time'
import { cn } from '@/lib/utils'
import { AURORA_MEDIUM_PANEL, AURORA_MUTED_LABEL } from '@/components/gateway/gateway-theme'
import { InstallDialog } from './install-dialog'
import { RegistryStatusBadge } from './registry-status-badge'
import { deleteServerLocalMetadata, setServerLocalMetadata } from '@/lib/api/mcpregistry-client'
import { TextSurface } from '@/components/ui/text-surface'
import type {
  EnvironmentVariable,
  Header,
  Icon as RegistryIcon,
  LabRegistryMetadata,
  NormalizedServerJSON,
  Package as RegistryPackage,
  PackageArgument,
  RegistryExtensions,
  RuntimeArgument,
  Transport,
} from '@/lib/types/registry'

interface ServerDetailPanelProps {
  server: NormalizedServerJSON | null
  extensions?: RegistryExtensions | null
  labMetadata?: LabRegistryMetadata | null
  onLabMetadataChange?: (metadata: LabRegistryMetadata | null) => void
  onClose: () => void
}

const MAX_SCHEMA_RESPONSE_BYTES = 256 * 1024
const textEncoder = new TextEncoder()
const EMPTY_LAB_METADATA_JSON = '{\n  "curation": {\n    "featured": false,\n    "tags": []\n  }\n}'

export function ServerDetailPanel({ server, extensions, labMetadata, onLabMetadataChange, onClose }: ServerDetailPanelProps) {
  const open = server !== null

  return (
    <Dialog open={open} onOpenChange={(v) => { if (!v) onClose() }}>
      <DialogContent className="flex max-h-[90vh] w-full flex-col gap-0 overflow-hidden p-0 sm:max-w-3xl">
        {server && (
          <PanelBody
            key={server.name}
            server={server}
            extensions={extensions ?? null}
            labMetadata={labMetadata ?? null}
            onLabMetadataChange={onLabMetadataChange}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}

function PanelBody({
  server,
  extensions,
  labMetadata,
  onLabMetadataChange,
}: {
  server: NormalizedServerJSON
  extensions: RegistryExtensions | null
  labMetadata: LabRegistryMetadata | null
  onLabMetadataChange?: (metadata: LabRegistryMetadata | null) => void
}) {
  const displayName = server.title ?? server.name
  const { remotes, icons, packages } = server
  const isHTTP = remotes.some((r) => r.type === 'streamable-http' || r.type === 'sse')
  const [installOpen, setInstallOpen] = useState(false)
  const [schemaOpen, setSchemaOpen] = useState(false)
  const [schemaContent, setSchemaContent] = useState<string | null>(null)
  const [schemaLoading, setSchemaLoading] = useState(false)
  const [schemaError, setSchemaError] = useState<string | null>(null)
  const [editableLabMetadata, setEditableLabMetadata] = useState<LabRegistryMetadata | null>(labMetadata)
  const [labMetadataDraft, setLabMetadataDraft] = useState(
    labMetadata ? JSON.stringify(labMetadata, null, 2) : EMPTY_LAB_METADATA_JSON,
  )
  const [labMetadataSaving, setLabMetadataSaving] = useState(false)
  const [labMetadataError, setLabMetadataError] = useState<string | null>(null)
  const [advancedMetadataOpen, setAdvancedMetadataOpen] = useState(false)

  useEffect(() => {
    setEditableLabMetadata(labMetadata)
    setLabMetadataDraft(
      labMetadata ? JSON.stringify(labMetadata, null, 2) : EMPTY_LAB_METADATA_JSON,
    )
    setLabMetadataError(null)
  }, [labMetadata, server.name])

  async function toggleSchema() {
    if (!schemaHref) return
    if (schemaContent !== null || schemaError !== null) {
      setSchemaOpen((v) => !v)
      return
    }
    setSchemaOpen(true)
    setSchemaLoading(true)
    setSchemaError(null)
    try {
      const res = await fetch(schemaHref)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const contentLength = Number(res.headers.get('content-length') ?? '0')
      if (contentLength > MAX_SCHEMA_RESPONSE_BYTES) {
        throw new Error(`Schema exceeds ${MAX_SCHEMA_RESPONSE_BYTES} bytes`)
      }
      const text = await res.text()
      if (textEncoder.encode(text).byteLength > MAX_SCHEMA_RESPONSE_BYTES) {
        throw new Error(`Schema exceeds ${MAX_SCHEMA_RESPONSE_BYTES} bytes`)
      }
      try {
        setSchemaContent(JSON.stringify(JSON.parse(text), null, 2))
      } catch {
        setSchemaContent(text)
      }
    } catch (err) {
      setSchemaError(err instanceof Error ? err.message : 'Failed to load schema')
    } finally {
      setSchemaLoading(false)
    }
  }
  const primaryIcon = icons[0] ?? null
  const extraIcons = icons.slice(1)
  const repoHref = safeHref(server.repository?.url)
  const ghAvatar = githubAvatarFromRepoUrl(server.repository?.url)
  const primaryIconHref = safeHref(primaryIcon?.src)
  const headerAvatarSrc = ghAvatar ?? primaryIconHref ?? null
  const websiteHref = safeHref(server.websiteUrl)
  const schemaHref = safeHref(server.$schema)
  const schemaPanelId = `schema-viewer-${server.name.replace(/[^a-zA-Z0-9_-]/g, '-')}`
  const status = extensions?.status ?? null
  const statusMessage = extensions?.statusMessage ?? null
  const displayedLabMetadata = editableLabMetadata
  const canonicalLabMetadataDraft = displayedLabMetadata
    ? JSON.stringify(displayedLabMetadata, null, 2)
    : EMPTY_LAB_METADATA_JSON
  const labMetadataDirty = labMetadataDraft !== canonicalLabMetadataDraft
  const editableLabMetadataView = editableLabMetadata ?? createDefaultLabMetadata()

  function updateLabMetadata(mutator: (current: LabRegistryMetadata) => LabRegistryMetadata) {
    setEditableLabMetadata((currentValue) => {
      const next = mutator(normalizeLabMetadata(currentValue))
      setLabMetadataDraft(JSON.stringify(next, null, 2))
      return next
    })
    setLabMetadataError(null)
  }

  function handleLabMetadataDraftChange(nextDraft: string) {
    setLabMetadataDraft(nextDraft)
    try {
      const parsed = JSON.parse(nextDraft) as LabRegistryMetadata
      if (parsed && !Array.isArray(parsed) && typeof parsed === 'object') {
        setEditableLabMetadata(parsed)
      }
    } catch {
      // Keep the last valid structured state while the advanced editor is mid-edit.
    }
  }

  async function saveLabMetadata() {
    setLabMetadataSaving(true)
    setLabMetadataError(null)
    try {
      const parsed = JSON.parse(labMetadataDraft) as LabRegistryMetadata
      if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
        throw new Error('Metadata must be a JSON object')
      }
      const result = await setServerLocalMetadata(
        server.name,
        parsed,
        { version: server.version, updated_by: 'gateway-admin' },
      )
      setEditableLabMetadata(result.metadata)
      setLabMetadataDraft(JSON.stringify(result.metadata, null, 2))
      onLabMetadataChange?.(result.metadata)
    } catch (error) {
      setLabMetadataError(error instanceof Error ? error.message : 'Failed to save metadata')
    } finally {
      setLabMetadataSaving(false)
    }
  }

  async function clearLabMetadata() {
    setLabMetadataSaving(true)
    setLabMetadataError(null)
    try {
      await deleteServerLocalMetadata(server.name, server.version)
      setEditableLabMetadata(null)
      setLabMetadataDraft(EMPTY_LAB_METADATA_JSON)
      onLabMetadataChange?.(null)
    } catch (error) {
      setLabMetadataError(error instanceof Error ? error.message : 'Failed to delete metadata')
    } finally {
      setLabMetadataSaving(false)
    }
  }

  return (
    <>
      <DialogHeader className="shrink-0 space-y-0 border-b border-aurora-border-strong/60 px-6 py-5">
        <div className="flex items-start gap-4">
          <div className="flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-aurora-2 border border-aurora-border-strong/60 bg-aurora-control-surface">
            {headerAvatarSrc ? (
              <>
                <img
                  src={headerAvatarSrc}
                  alt=""
                  className="size-full object-cover"
                  referrerPolicy="no-referrer"
                  loading="lazy"
                  onError={(e) => {
                    const img = e.currentTarget
                    if (ghAvatar && primaryIconHref && img.dataset.fallbackApplied !== 'true') {
                      img.dataset.fallbackApplied = 'true'
                      img.src = primaryIconHref
                      return
                    }
                    img.style.display = 'none'
                    ;(img.nextElementSibling as HTMLElement | null)?.removeAttribute('style')
                  }}
                />
                <Package className="size-6 text-aurora-text-muted" style={{ display: 'none' }} />
              </>
            ) : (
              <Package className="size-6 text-aurora-text-muted" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <DialogTitle className="text-base font-semibold leading-tight text-aurora-text-primary">
              {displayName}
            </DialogTitle>
            <div className="mt-0.5 flex items-center gap-1.5">
              <p className="break-all font-mono text-xs text-aurora-text-muted">{server.name}</p>
              <CopyButton value={server.name} label="Copy server name" />
            </div>
            <div className="mt-2 flex flex-wrap gap-2">
              <span className="rounded-full border border-aurora-border-strong/60 px-2 py-0.5 text-xs text-aurora-text-muted">
                v{server.version}
              </span>
              <span
                className={cn(
                  'rounded-full px-2 py-0.5 text-xs font-medium',
                  isHTTP
                    ? 'bg-aurora-accent-strong/15 text-aurora-accent-strong'
                    : 'bg-aurora-border-strong/40 text-aurora-text-muted',
                )}
              >
                {isHTTP ? 'HTTP' : 'stdio only'}
              </span>
              {extensions?.isLatest && (
                <span className="inline-flex items-center gap-1 rounded-full border border-aurora-accent-primary/30 bg-aurora-accent-primary/10 px-2 py-0.5 text-xs font-medium text-aurora-accent-primary">
                  <CheckCircle2 className="size-3" /> Latest
                </span>
              )}
              {extensions?.updatedAt && <TimestampPill iso={extensions.updatedAt} />}
              <RegistryStatusBadge status={status} />
            </div>
            {statusMessage && (
              <p className="mt-2 text-xs leading-relaxed text-aurora-text-muted">
                {statusMessage}
              </p>
            )}
          </div>
        </div>
      </DialogHeader>

      <div className="flex-1 space-y-6 overflow-y-auto px-6 py-5">
        {/* Description — untrusted registry data, do not use dangerouslySetInnerHTML */}
        <Section label="Description">
          <p className="whitespace-pre-wrap text-sm leading-relaxed text-aurora-text-muted">
            {server.description}
          </p>
        </Section>

        {(repoHref || websiteHref || schemaHref || server.repository?.id || server.repository?.subfolder) && (
          <Section label="Links">
            <div className="flex flex-wrap gap-3">
              {repoHref && (
                <a
                  href={repoHref}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                >
                  <ExternalLink className="size-3.5" />
                  Repository
                  {server.repository?.source && (
                    <span className="text-xs text-aurora-text-muted">({server.repository.source})</span>
                  )}
                </a>
              )}
              {websiteHref && (
                <a
                  href={websiteHref}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                >
                  <Globe className="size-3.5" />
                  Website
                </a>
              )}
              {schemaHref && (
                <button
                  type="button"
                  onClick={toggleSchema}
                  aria-expanded={schemaOpen}
                  aria-controls={schemaPanelId}
                  className="inline-flex items-center gap-1.5 text-sm text-aurora-accent-strong hover:underline"
                  title={server.$schema ?? undefined}
                >
                  <FileCode className="size-3.5" />
                  Schema
                  {schemaOpen
                    ? <ChevronDown className="size-3" />
                    : <ChevronRight className="size-3" />}
                </button>
              )}
            </div>
            {(server.repository?.id || server.repository?.subfolder) && (
              <dl className={cn(AURORA_MEDIUM_PANEL, 'mt-2 grid grid-cols-1 gap-x-6 gap-y-2 p-3 text-xs sm:grid-cols-2')}>
                {server.repository?.id && <MetaRow label="Repository ID" value={server.repository.id} mono copy />}
                {server.repository?.subfolder && <MetaRow label="Subfolder" value={server.repository.subfolder} mono />}
              </dl>
            )}
          </Section>
        )}

        {schemaOpen && schemaHref && (
          <SchemaViewer
            id={schemaPanelId}
            url={schemaHref}
            content={schemaContent}
            loading={schemaLoading}
            error={schemaError}
          />
        )}

        {extraIcons.length > 0 && (
          <Section label={`Icons (${icons.length})`}>
            <div className="flex flex-wrap gap-2">
              {icons.map((icon, i) => <IconChip key={i} icon={icon} />)}
            </div>
          </Section>
        )}

        {remotes.length > 0 && (
          <Section label={`Remote transports (${remotes.length})`}>
            <div className="space-y-2">
              {remotes.map((r, i) => <RemoteRow key={i} transport={r} />)}
            </div>
          </Section>
        )}

        {remotes.length === 0 && (
          <div className={cn(AURORA_MEDIUM_PANEL, 'p-4 text-sm text-aurora-text-muted')}>
            This server uses stdio transport only and cannot be added to a gateway.
          </div>
        )}

        {packages.length > 0 && (
          <Section label={`Packages (${packages.length})`}>
            <div className="space-y-3">
              {packages.map((p, i) => <PackageCard key={i} pkg={p} />)}
            </div>
          </Section>
        )}

        {extensions && (
          <Section label="Registry metadata">
            <dl className={cn(AURORA_MEDIUM_PANEL, 'grid grid-cols-1 gap-x-6 gap-y-2 p-4 text-xs sm:grid-cols-2')}>
              <TimeRow label="Published" iso={extensions.publishedAt} />
              <TimeRow label="Updated" iso={extensions.updatedAt} />
              <TimeRow label="Status changed" iso={extensions.statusChangedAt} />
              <MetaRow label="Status" value={extensions.status} />
              <MetaRow label="Is latest" value={extensions.isLatest ? 'yes' : 'no'} />
            </dl>
          </Section>
        )}

        {displayedLabMetadata && (
          <Section label="Lab metadata">
            <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-4 text-xs')}>
              <dl className="grid grid-cols-1 gap-x-6 gap-y-2 sm:grid-cols-2">
                <BooleanRow label="Featured" value={displayedLabMetadata.curation?.featured} />
                <BooleanRow label="Hidden" value={displayedLabMetadata.curation?.hidden} />
                <BooleanRow label="Reviewed" value={displayedLabMetadata.trust?.reviewed} />
                <BooleanRow label="Source verified" value={displayedLabMetadata.trust?.source_verified} />
                <BooleanRow label="Maintainer known" value={displayedLabMetadata.trust?.maintainer_known} />
                <BooleanRow label="Install tested" value={displayedLabMetadata.quality?.install_tested} />
                <BooleanRow label="SSRF reviewed" value={displayedLabMetadata.security?.ssrf_reviewed} />
                <BooleanRow label="Permissions reviewed" value={displayedLabMetadata.security?.permissions_reviewed} />
                <BooleanRow label="Secrets reviewed" value={displayedLabMetadata.security?.secrets_reviewed} />
                <BooleanRow label="Works in Lab" value={displayedLabMetadata.ux?.works_in_lab} />
                <BooleanRow label="Recommended" value={displayedLabMetadata.ux?.recommended_for_homelab} />
                <MetaRow label="Transport score" value={displayedLabMetadata.quality?.transport_score ?? null} />
                <MetaRow label="Setup difficulty" value={displayedLabMetadata.ux?.setup_difficulty ?? null} />
                <TimeRow label="Reviewed at" iso={displayedLabMetadata.trust?.reviewed_at ?? null} />
                <TimeRow label="Install tested at" iso={displayedLabMetadata.quality?.last_install_tested_at ?? null} />
                <TimeRow label="Updated at" iso={displayedLabMetadata.audit?.updated_at ?? null} />
                <MetaRow label="Updated by" value={displayedLabMetadata.audit?.updated_by ?? null} />
              </dl>
              {displayedLabMetadata.curation?.tags && displayedLabMetadata.curation.tags.length > 0 && (
                <div className="space-y-1">
                  <p className={AURORA_MUTED_LABEL}>Tags</p>
                  <div className="flex flex-wrap gap-2">
                    {displayedLabMetadata.curation.tags.map((tag) => (
                      <span
                        key={tag}
                        className="rounded-full border border-aurora-border-strong/60 px-2 py-0.5 text-[11px] text-aurora-text-muted"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              {displayedLabMetadata.curation?.notes && (
                <div className="space-y-1">
                  <p className={AURORA_MUTED_LABEL}>Notes</p>
                  <p className="whitespace-pre-wrap text-xs leading-relaxed text-aurora-text-muted">
                    {displayedLabMetadata.curation.notes}
                  </p>
                </div>
              )}
              <div className="space-y-1">
                <p className={AURORA_MUTED_LABEL}>Raw metadata</p>
                <div className="aurora-scrollbar max-h-64 overflow-auto rounded border border-aurora-border-strong/40 bg-aurora-page-bg">
                  <pre className="w-max min-w-full p-3 font-mono text-xs leading-relaxed text-aurora-text-muted">
                    <JsonHighlight content={JSON.stringify(displayedLabMetadata, null, 2)} />
                  </pre>
                </div>
              </div>
            </div>
          </Section>
        )}

        <Section label="Edit Lab metadata">
          <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-4')}>
            <StructuredMetadataEditor
              metadata={editableLabMetadataView}
              onChange={updateLabMetadata}
            />
            <div className="rounded-aurora-2 border border-aurora-border-strong/40 bg-aurora-page-bg">
              <button
                type="button"
                className="flex w-full items-center justify-between gap-2 px-4 py-3 text-left"
                onClick={() => setAdvancedMetadataOpen((value) => !value)}
                aria-expanded={advancedMetadataOpen}
              >
                <div>
                  <p className="text-sm font-medium text-aurora-text-primary">Advanced JSON</p>
                  <p className="mt-1 text-xs text-aurora-text-muted">
                    Use CodeMirror for fields outside the structured form.
                  </p>
                </div>
                {advancedMetadataOpen ? (
                  <ChevronDown className="size-4 text-aurora-text-muted" />
                ) : (
                  <ChevronRight className="size-4 text-aurora-text-muted" />
                )}
              </button>
              {advancedMetadataOpen && (
                <div className="border-t border-aurora-border-strong/40 p-4 pt-3">
                  <div className="h-72">
                    <TextSurface
                      path={`registry/${server.name.replaceAll('/', '__')}.lab-meta.json`}
                      value={labMetadataDraft}
                      mode="edit"
                      language="json"
                      dirty={labMetadataDirty}
                      onChange={handleLabMetadataDraftChange}
                      onSave={() => void saveLabMetadata()}
                    />
                  </div>
                </div>
              )}
            </div>
            {labMetadataError && (
              <p className="text-xs text-aurora-error">{labMetadataError}</p>
            )}
            <div className="flex flex-wrap gap-2">
              <Button type="button" size="sm" onClick={() => void saveLabMetadata()} disabled={labMetadataSaving}>
                {labMetadataSaving ? 'Saving…' : 'Save metadata'}
              </Button>
              <Button type="button" size="sm" variant="outline" onClick={() => void clearLabMetadata()} disabled={labMetadataSaving}>
                Delete metadata
              </Button>
            </div>
          </div>
        </Section>
      </div>

      <div className="shrink-0 border-t border-aurora-border-strong/60 px-6 py-4">
        {isHTTP ? (
          <Button className="w-full" onClick={() => setInstallOpen(true)}>
            Install to Gateway
          </Button>
        ) : (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="w-full">
                  <Button disabled className="w-full" tabIndex={-1}>
                    Install to Gateway
                  </Button>
                </span>
              </TooltipTrigger>
              <TooltipContent>
                This server uses stdio transport only and cannot be installed to a gateway
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </div>

      <InstallDialog
        server={installOpen ? server : null}
        onClose={() => setInstallOpen(false)}
      />
    </>
  )
}

function StructuredMetadataEditor({
  metadata,
  onChange,
}: {
  metadata: LabRegistryMetadata
  onChange: (mutator: (current: LabRegistryMetadata) => LabRegistryMetadata) => void
}) {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-2">
        <ToggleField
          label="Featured"
          description="Highlight this server in the registry list."
          checked={metadata.curation?.featured ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            curation: { ...current.curation, featured: checked },
          }))}
        />
        <ToggleField
          label="Hidden"
          description="Hide this server from normal curation views."
          checked={metadata.curation?.hidden ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            curation: { ...current.curation, hidden: checked },
          }))}
        />
        <ToggleField
          label="Reviewed"
          description="Marks that Lab has reviewed this server."
          checked={metadata.trust?.reviewed ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            trust: { ...current.trust, reviewed: checked },
          }))}
        />
        <ToggleField
          label="Source verified"
          description="Repository and package source were checked."
          checked={metadata.trust?.source_verified ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            trust: { ...current.trust, source_verified: checked },
          }))}
        />
        <ToggleField
          label="Maintainer known"
          description="Maintainer identity is known and stable."
          checked={metadata.trust?.maintainer_known ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            trust: { ...current.trust, maintainer_known: checked },
          }))}
        />
        <ToggleField
          label="Install tested"
          description="Install flow was tested in Lab."
          checked={metadata.quality?.install_tested ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            quality: { ...current.quality, install_tested: checked },
          }))}
        />
        <ToggleField
          label="SSRF reviewed"
          description="Remote URL and network behavior were reviewed."
          checked={metadata.security?.ssrf_reviewed ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            security: { ...current.security, ssrf_reviewed: checked },
          }))}
        />
        <ToggleField
          label="Permissions reviewed"
          description="Requested permissions were checked."
          checked={metadata.security?.permissions_reviewed ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            security: { ...current.security, permissions_reviewed: checked },
          }))}
        />
        <ToggleField
          label="Secrets reviewed"
          description="Secret handling and auth inputs were reviewed."
          checked={metadata.security?.secrets_reviewed ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            security: { ...current.security, secrets_reviewed: checked },
          }))}
        />
        <ToggleField
          label="Works in Lab"
          description="Confirmed working in the Lab product flow."
          checked={metadata.ux?.works_in_lab ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            ux: { ...current.ux, works_in_lab: checked },
          }))}
        />
        <ToggleField
          label="Recommended"
          description="Recommended for homelab operators."
          checked={metadata.ux?.recommended_for_homelab ?? false}
          onCheckedChange={(checked) => onChange((current) => ({
            ...current,
            ux: { ...current.ux, recommended_for_homelab: checked },
          }))}
        />
      </div>

      <div className="grid gap-3 md:grid-cols-2">
        <FormField label="Tags" description="Comma-separated curation tags.">
          <Input
            value={(metadata.curation?.tags ?? []).join(', ')}
            onChange={(event) => {
              const tags = event.target.value
                .split(',')
                .map((value) => value.trim())
                .filter(Boolean)
              onChange((current) => ({
                ...current,
                curation: { ...current.curation, tags },
              }))
            }}
            placeholder="recommended, stable, homelab"
          />
        </FormField>
        <FormField label="Transport score" description="Operational transport quality summary.">
          <Input
            value={metadata.quality?.transport_score ?? ''}
            onChange={(event) => onChange((current) => ({
              ...current,
              quality: { ...current.quality, transport_score: stringOrUndefined(event.target.value) },
            }))}
            placeholder="good"
          />
        </FormField>
        <FormField label="Setup difficulty" description="Operator setup complexity.">
          <Input
            value={metadata.ux?.setup_difficulty ?? ''}
            onChange={(event) => onChange((current) => ({
              ...current,
              ux: { ...current.ux, setup_difficulty: stringOrUndefined(event.target.value) },
            }))}
            placeholder="easy"
          />
        </FormField>
        <FormField label="Reviewed at" description="RFC3339 timestamp.">
          <Input
            value={metadata.trust?.reviewed_at ?? ''}
            onChange={(event) => onChange((current) => ({
              ...current,
              trust: { ...current.trust, reviewed_at: stringOrUndefined(event.target.value) },
            }))}
            placeholder="2026-04-23T15:00:00Z"
          />
        </FormField>
        <FormField label="Install tested at" description="RFC3339 timestamp.">
          <Input
            value={metadata.quality?.last_install_tested_at ?? ''}
            onChange={(event) => onChange((current) => ({
              ...current,
              quality: { ...current.quality, last_install_tested_at: stringOrUndefined(event.target.value) },
            }))}
            placeholder="2026-04-23T15:00:00Z"
          />
        </FormField>
      </div>

      <FormField label="Notes" description="Freeform operator notes for this server.">
        <Textarea
          value={metadata.curation?.notes ?? ''}
          onChange={(event) => onChange((current) => ({
            ...current,
            curation: { ...current.curation, notes: stringOrUndefined(event.target.value) },
          }))}
          placeholder="Known caveats, operator guidance, or review notes."
          rows={4}
        />
      </FormField>
    </div>
  )
}

function FormField({
  label,
  description,
  children,
}: {
  label: string
  description?: string
  children: React.ReactNode
}) {
  return (
    <div className="space-y-2">
      <div className="space-y-1">
        <Label className="text-xs font-medium text-aurora-text-primary">{label}</Label>
        {description && <p className="text-xs text-aurora-text-muted">{description}</p>}
      </div>
      {children}
    </div>
  )
}

function ToggleField({
  label,
  description,
  checked,
  onCheckedChange,
}: {
  label: string
  description: string
  checked: boolean
  onCheckedChange: (checked: boolean) => void
}) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-aurora-2 border border-aurora-border-strong/40 bg-aurora-page-bg p-3">
      <div className="space-y-1">
        <Label className="text-xs font-medium text-aurora-text-primary">{label}</Label>
        <p className="text-xs leading-relaxed text-aurora-text-muted">{description}</p>
      </div>
      <Switch checked={checked} onCheckedChange={onCheckedChange} />
    </div>
  )
}

function createDefaultLabMetadata(): LabRegistryMetadata {
  return {
    curation: {
      featured: false,
      tags: [],
    },
  }
}

function normalizeLabMetadata(metadata: LabRegistryMetadata | null): LabRegistryMetadata {
  if (metadata) return metadata
  return createDefaultLabMetadata()
}

function stringOrUndefined(value: string): string | undefined {
  const trimmed = value.trim()
  return trimmed ? trimmed : undefined
}

function BooleanRow({ label, value }: { label: string; value?: boolean | null }) {
  if (value == null) return null
  return <MetaRow label={label} value={value ? 'yes' : 'no'} />
}

type JsonTokenType = 'key' | 'string' | 'number' | 'boolean' | 'null' | 'punctuation' | 'whitespace'

const JSON_TOKEN_CLASS: Record<JsonTokenType, string> = {
  key: 'text-aurora-accent-strong',
  string: 'text-aurora-success',
  number: 'text-aurora-warn',
  boolean: 'text-aurora-accent-primary',
  null: 'text-aurora-text-muted',
  punctuation: 'text-aurora-text-muted',
  whitespace: '',
}

function JsonHighlight({ content }: { content: string }) {
  const parts: React.ReactNode[] = []
  const jsonTokenRe = /"(?:[^"\\]|\\.)*"|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?|true|false|null|[{}[\]:,]|\s+|./g

  for (const match of content.matchAll(jsonTokenRe)) {
    const token = match[0]
    let type: JsonTokenType = 'punctuation'

    if (/^\s+$/.test(token)) {
      type = 'whitespace'
    } else if (token === 'true' || token === 'false') {
      type = 'boolean'
    } else if (token === 'null') {
      type = 'null'
    } else if (/^-?\d/.test(token)) {
      type = 'number'
    } else if (token.startsWith('"')) {
      // Key strings are immediately followed by ':' after optional whitespace
      const rest = content.slice(match.index! + token.length).trimStart()
      type = rest.startsWith(':') ? 'key' : 'string'
    }

    const cls = JSON_TOKEN_CLASS[type]
    parts.push(cls ? <span key={match.index} className={cls}>{token}</span> : token)
  }

  return <>{parts}</>
}

function SchemaViewer({
  id,
  url,
  content,
  loading,
  error,
}: {
  id?: string
  url: string
  content: string | null
  loading: boolean
  error: string | null
}) {
  return (
    <div id={id} className={cn(AURORA_MEDIUM_PANEL, 'space-y-2 p-3')}>
      <div className="flex items-center justify-between gap-2">
        <p className="break-all font-mono text-[11px] text-aurora-text-muted">{url}</p>
        {content && <CopyButton value={content} label="Copy schema" />}
      </div>
      {loading && (
        <div className="flex items-center gap-2 py-4 text-xs text-aurora-text-muted">
          <Spinner className="size-3.5" />
          Loading schema…
        </div>
      )}
      {error && (
        <p className="py-2 text-xs text-aurora-error">{error}</p>
      )}
      {content && (
        <div className="aurora-scrollbar max-h-80 overflow-auto rounded border border-aurora-border-strong/40 bg-aurora-page-bg">
          <pre className="w-max min-w-full p-3 font-mono text-xs leading-relaxed text-aurora-text-muted">
            <JsonHighlight content={content} />
          </pre>
        </div>
      )}
    </div>
  )
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      {children}
    </div>
  )
}

function MetaRow({
  label,
  value,
  mono,
  copy,
}: {
  label: string
  value?: string | null
  mono?: boolean
  copy?: boolean
}) {
  if (!value) return null
  return (
    <div className="flex flex-col gap-0.5">
      <dt className={AURORA_MUTED_LABEL}>{label}</dt>
      <dd className={cn('flex items-center gap-1.5 text-xs text-aurora-text-muted', mono && 'font-mono')}>
        <span className="break-all">{value}</span>
        {copy && <CopyButton value={value} label={`Copy ${label.toLowerCase()}`} />}
      </dd>
    </div>
  )
}

function TimeRow({ label, iso }: { label: string; iso?: string | null }) {
  const local = formatLocalDateTimePrecise(iso)
  const utc = formatUtcTooltip(iso)
  if (!local) return null
  return (
    <div className="flex flex-col gap-0.5">
      <dt className={AURORA_MUTED_LABEL}>{label}</dt>
      <dd className="font-mono text-xs text-aurora-text-muted" title={utc ?? undefined}>
        {local}
      </dd>
    </div>
  )
}

function TimestampPill({ iso }: { iso: string }) {
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return null
  const tooltip = [formatLocalDateTimePrecise(iso), formatUtcTooltip(iso)]
    .filter(Boolean)
    .join(' · ')
  return (
    <span
      title={tooltip}
      className="cursor-default rounded-full border border-aurora-border-strong/40 px-2 py-0.5 text-xs text-aurora-text-muted"
    >
      {formatDistanceToNow(d, { addSuffix: true })}
    </span>
  )
}

function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={async (e) => {
        e.stopPropagation()
        try {
          await navigator.clipboard.writeText(value)
          setCopied(true)
          setTimeout(() => setCopied(false), 1200)
        } catch {
          // clipboard access denied
        }
      }}
      className="inline-flex size-5 shrink-0 items-center justify-center rounded text-aurora-text-muted hover:bg-aurora-border-strong/40 hover:text-aurora-text-primary"
    >
      {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
    </button>
  )
}

function IconChip({ icon }: { icon: RegistryIcon }) {
  const href = safeHref(icon.src)
  const label = [icon.theme, icon.sizes?.join(', '), icon.mimeType]
    .filter(Boolean)
    .join(' · ')
  return (
    <div
      className={cn(AURORA_MEDIUM_PANEL, 'flex items-center gap-2 px-2 py-1.5 text-xs')}
      title={icon.src}
    >
      {href ? (
        <img
          src={href}
          alt=""
          className="size-6 rounded object-contain"
          referrerPolicy="no-referrer"
          loading="lazy"
          onError={(e) => { e.currentTarget.style.display = 'none' }}
        />
      ) : (
        <Package className="size-4 text-aurora-text-muted" />
      )}
      <span className="font-mono text-[11px] text-aurora-text-muted">
        {label || 'icon'}
      </span>
    </div>
  )
}

function RemoteRow({ transport }: { transport: Transport }) {
  const headers: Header[] = transport.headers ?? []
  const variables = transport.variables ?? null
  const hasVariables = variables && Object.keys(variables).length > 0

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-2 p-3 text-sm')}>
      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-full bg-aurora-accent-strong/15 px-2 py-0.5 text-xs font-medium text-aurora-accent-strong">
          {transport.type}
        </span>
      </div>
      {transport.url && (
        <div className="flex items-center gap-1.5">
          <p className="break-all font-mono text-xs text-aurora-text-muted">{transport.url}</p>
          <CopyButton value={transport.url} label="Copy URL" />
        </div>
      )}
      {headers.length > 0 && (
        <div className="space-y-1">
          <p className={AURORA_MUTED_LABEL}>Headers ({headers.length})</p>
          <ul className="space-y-0.5">
            {headers.map((h, i) => (
              <li key={i} className="font-mono text-xs text-aurora-text-muted">
                <span className="text-aurora-text-primary">{h.name}:</span> {h.value}
              </li>
            ))}
          </ul>
        </div>
      )}
      {hasVariables && (
        <div className="space-y-1">
          <p className={AURORA_MUTED_LABEL}>Variables</p>
          <pre className="overflow-x-auto rounded border border-aurora-border-strong/40 bg-aurora-page-bg p-2 font-mono text-xs text-aurora-text-muted">
            {JSON.stringify(variables, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}

function PackageCard({ pkg }: { pkg: RegistryPackage }) {
  const runtimeArgs = pkg.runtimeArguments ?? []
  const packageArgs = pkg.packageArguments ?? []
  const envVars = pkg.environmentVariables ?? []

  return (
    <div className={cn(AURORA_MEDIUM_PANEL, 'space-y-3 p-3 text-sm')}>
      <div className="space-y-1">
        <div className="flex flex-wrap items-baseline gap-2">
          <span className="font-medium text-aurora-text-primary">{pkg.identifier}</span>
          <CopyButton value={pkg.identifier} label="Copy identifier" />
          {pkg.version && (
            <span className="text-xs text-aurora-text-muted">v{pkg.version}</span>
          )}
          <span className="rounded-full border border-aurora-border-strong/60 px-2 py-0.5 text-[11px] text-aurora-text-muted">
            {pkg.registryType}
          </span>
          {pkg.runtimeHint && (
            <span className="rounded-full bg-aurora-border-strong/30 px-2 py-0.5 text-[11px] text-aurora-text-muted">
              {pkg.runtimeHint}
            </span>
          )}
        </div>
        {pkg.registryBaseUrl && (
          <div className="flex items-center gap-1.5">
            <p className="break-all font-mono text-[11px] text-aurora-text-muted">
              {pkg.registryBaseUrl}
            </p>
            <CopyButton value={pkg.registryBaseUrl} label="Copy registry URL" />
          </div>
        )}
        {pkg.fileSha256 && (
          <div className="flex items-center gap-1.5">
            <p className="break-all font-mono text-[11px] text-aurora-text-muted">
              sha256: {pkg.fileSha256}
            </p>
            <CopyButton value={pkg.fileSha256} label="Copy sha256" />
          </div>
        )}
      </div>

      <div className="space-y-1">
        <p className={AURORA_MUTED_LABEL}>Transport</p>
        <div className="flex items-center gap-1.5 font-mono text-xs text-aurora-text-muted">
          <span className="text-aurora-text-primary">{pkg.transport.type}</span>
          {pkg.transport.url && (
            <>
              <span>·</span>
              <span className="break-all">{pkg.transport.url}</span>
              <CopyButton value={pkg.transport.url} label="Copy URL" />
            </>
          )}
        </div>
      </div>

      {runtimeArgs.length > 0 && <ArgsList label={`Runtime arguments (${runtimeArgs.length})`} items={runtimeArgs} renderItem={renderRuntimeArg} />}
      {packageArgs.length > 0 && <ArgsList label={`Package arguments (${packageArgs.length})`} items={packageArgs} renderItem={renderPackageArg} />}
      {envVars.length > 0 && <ArgsList label={`Environment variables (${envVars.length})`} items={envVars} renderItem={renderEnvVar} />}
    </div>
  )
}

function ArgsList<T>({
  label,
  items,
  renderItem,
}: {
  label: string
  items: T[]
  renderItem: (item: T, i: number) => React.ReactNode
}) {
  return (
    <div className="space-y-1">
      <p className={AURORA_MUTED_LABEL}>{label}</p>
      <ul className="space-y-1">{items.map(renderItem)}</ul>
    </div>
  )
}

function renderRuntimeArg(arg: RuntimeArgument, i: number) {
  return (
    <li key={i} className="font-mono text-xs text-aurora-text-muted">
      <span className="text-aurora-text-primary">{arg.name}</span>
      {arg.value && <> = {arg.value}</>}
    </li>
  )
}

function renderPackageArg(arg: PackageArgument, i: number) {
  return (
    <li key={i} className="text-xs">
      <div className="flex items-baseline gap-2">
        <span className="font-mono text-aurora-text-primary">{arg.value}</span>
        {arg.isRequired && (
          <span className="rounded-full bg-aurora-warn/15 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider text-aurora-warn">
            required
          </span>
        )}
      </div>
      {arg.description && (
        <p className="mt-0.5 text-aurora-text-muted">{arg.description}</p>
      )}
    </li>
  )
}

function renderEnvVar(env: EnvironmentVariable, i: number) {
  return (
    <li key={i} className="text-xs">
      <div className="flex items-baseline gap-2">
        <span className="font-mono text-aurora-text-primary">{env.name}</span>
        {env.required && (
          <span className="rounded-full bg-aurora-warn/15 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wider text-aurora-warn">
            required
          </span>
        )}
      </div>
      {env.description && (
        <p className="mt-0.5 text-aurora-text-muted">{env.description}</p>
      )}
    </li>
  )
}
