'use client'

import { useRef, useState } from 'react'
import { ShoppingBag } from 'lucide-react'
import { Dialog, DialogContent, DialogTitle } from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import type { Marketplace } from '@/lib/types/marketplace'

interface AddMarketplaceModalProps {
  open: boolean
  onClose: () => void
  onAdd: (input: { repo?: string; url?: string; name?: string; autoUpdate: boolean }) => Promise<Marketplace | null>
}

export function AddMarketplaceModal({ open, onClose, onAdd }: AddMarketplaceModalProps) {
  const repoInputId = 'marketplace-repo'
  const urlInputId = 'marketplace-url'
  const nameInputId = 'marketplace-name'
  const autoUpdateInputId = 'marketplace-auto-update'
  const [repo, setRepo] = useState('')
  const [url, setUrl] = useState('')
  const [name, setName] = useState('')
  const [autoUpdate, setAutoUpdate] = useState(true)
  const [loading, setLoading] = useState(false)
  const isSubmittingRef = useRef(false)

  async function handleSubmit() {
    if (isSubmittingRef.current || loading) return
    if (!repo.trim() && !url.trim()) return
    isSubmittingRef.current = true
    setLoading(true)
    try {
      const result = await onAdd({
        repo: repo.trim() || undefined,
        url: url.trim() || undefined,
        name: name.trim() || undefined,
        autoUpdate,
      })
      if (result) {
        setRepo(''); setUrl(''); setName(''); setAutoUpdate(true)
        onClose()
      }
    } finally {
      isSubmittingRef.current = false
      setLoading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={v => { if (!v) onClose() }}>
      <DialogContent className="w-[520px] max-w-[calc(100vw-40px)] p-0 bg-aurora-panel-strong border-aurora-border-strong rounded-aurora-3 overflow-hidden gap-0">
        <DialogTitle className="sr-only">Add Marketplace</DialogTitle>

        <div className="px-7 pt-6 pb-5 border-b border-aurora-border-default bg-[linear-gradient(180deg,color-mix(in_srgb,var(--aurora-panel-strong)_80%,transparent),transparent)]">
          <div className="flex items-center gap-[10px] font-display text-[19px] font-extrabold tracking-[-0.02em] text-aurora-text-primary">
            <div className="w-8 h-8 rounded-[10px] flex-shrink-0 flex items-center justify-center text-aurora-accent-primary bg-[linear-gradient(135deg,color-mix(in_srgb,var(--aurora-accent-primary)_18%,transparent),color-mix(in_srgb,var(--aurora-accent-deep)_24%,transparent))] border border-[color-mix(in_srgb,var(--aurora-accent-primary)_20%,transparent)]">
              <ShoppingBag className="w-4 h-4" />
            </div>
            Add Marketplace
          </div>
          <p className="text-[13px] text-aurora-text-muted mt-1 leading-[1.5]">
            Connect a GitHub repo or git URL to browse its plugin catalogue.
          </p>
        </div>

        <div className="px-7 py-6 flex flex-col gap-[18px]">
          <div className="flex flex-col gap-[7px]">
            <label htmlFor={repoInputId} className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
              GitHub Repository
            </label>
            <input
              id={repoInputId}
              className="bg-aurora-control-surface border border-aurora-border-strong rounded-aurora-1 text-aurora-text-primary placeholder:text-aurora-text-muted/55 px-[14px] py-[10px] text-[13px] outline-none focus:border-aurora-accent-primary focus:shadow-[0_0_0_3px_var(--aurora-focus-ring)] transition-[border-color,box-shadow] shadow-[var(--aurora-shadow-inset)]"
              value={repo}
              onChange={e => setRepo(e.target.value)}
              placeholder="owner/repo — e.g. obra/superpowers-marketplace"
            />
            <span className="text-[11px] text-aurora-text-muted/60">Leave blank if providing a git URL below</span>
          </div>

          <div className="flex flex-col gap-[7px]">
            <label htmlFor={urlInputId} className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
              Or Git URL
            </label>
            <input
              id={urlInputId}
              className="bg-aurora-control-surface border border-aurora-border-strong rounded-aurora-1 text-aurora-text-primary placeholder:text-aurora-text-muted/55 px-[14px] py-[10px] text-[13px] outline-none focus:border-aurora-accent-primary focus:shadow-[0_0_0_3px_var(--aurora-focus-ring)] transition-[border-color,box-shadow] shadow-[var(--aurora-shadow-inset)]"
              value={url}
              onChange={e => setUrl(e.target.value)}
              placeholder="https://github.com/…/marketplace.git"
            />
          </div>

          <div className="flex flex-col gap-[7px]">
            <label htmlFor={nameInputId} className="text-[11px] font-bold uppercase tracking-[0.12em] text-aurora-text-muted">
              Marketplace Name{' '}
              <span className="text-[10px] font-normal normal-case tracking-normal opacity-60">(optional)</span>
            </label>
            <input
              id={nameInputId}
              className="bg-aurora-control-surface border border-aurora-border-strong rounded-aurora-1 text-aurora-text-primary placeholder:text-aurora-text-muted/55 px-[14px] py-[10px] text-[13px] outline-none focus:border-aurora-accent-primary focus:shadow-[0_0_0_3px_var(--aurora-focus-ring)] transition-[border-color,box-shadow] shadow-[var(--aurora-shadow-inset)]"
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="auto-detected from manifest"
            />
          </div>

          <div className="flex items-center justify-between bg-aurora-control-surface border border-aurora-border-strong rounded-aurora-1 px-[14px] py-3 shadow-[var(--aurora-shadow-inset)]">
            <div className="flex flex-col gap-0.5">
              <span className="text-[13px] font-medium text-aurora-text-primary">Auto-update</span>
              <span className="text-[11px] text-aurora-text-muted">Sync new plugins automatically</span>
            </div>
            <label htmlFor={autoUpdateInputId} className="relative w-9 h-5 flex-shrink-0 cursor-pointer">
              <input
                id={autoUpdateInputId}
                type="checkbox"
                className="sr-only peer"
                checked={autoUpdate}
                onChange={e => setAutoUpdate(e.target.checked)}
              />
              <div className="absolute inset-0 rounded-full bg-aurora-border-strong peer-checked:bg-aurora-accent-primary transition-colors duration-200" />
              <div className={cn(
                'absolute top-[3px] left-[3px] w-[14px] h-[14px] rounded-full bg-aurora-text-primary shadow-[0_1px_4px_color-mix(in_srgb,black_30%,transparent)] transition-transform duration-200',
                autoUpdate && 'translate-x-4',
              )} />
            </label>
          </div>
        </div>

        <div className="flex justify-end gap-2 px-7 py-4 pb-6 border-t border-aurora-border-default">
          <button
            onClick={onClose}
            className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer border border-transparent bg-transparent text-aurora-text-muted hover:bg-aurora-hover-bg hover:text-aurora-text-primary transition-all duration-150"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={loading || (!repo.trim() && !url.trim())}
            className="inline-flex items-center gap-1.5 px-[14px] py-1.5 rounded-lg font-sans text-[13px] font-semibold cursor-pointer bg-aurora-accent-primary text-aurora-page-bg hover:bg-aurora-accent-strong transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <ShoppingBag className="w-[14px] h-[14px]" />
            {loading ? 'Adding…' : 'Add Marketplace'}
          </button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
