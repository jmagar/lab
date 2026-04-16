'use client'

import type { LucideIcon } from 'lucide-react'
import {
  Bell,
  BookMarked,
  Bot,
  Boxes,
  Brain,
  Database,
  Film,
  FolderKanban,
  HardDrive,
  Library,
  RadioTower,
  Router,
  ScanSearch,
  Search,
  Server,
  ShieldEllipsis,
  Tv,
  Waypoints,
  Wifi,
} from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import type { SupportedService } from '@/lib/types/gateway'
import { cn } from '@/lib/utils'

interface LabServicePickerProps {
  selectedService: string
  services: SupportedService[]
  onSelect: (serviceKey: string) => void
}

const SERVICE_ICONS: Record<string, LucideIcon> = {
  apprise: Bell,
  arcane: Boxes,
  bytestash: FolderKanban,
  gotify: RadioTower,
  linkding: BookMarked,
  memos: Library,
  openai: Brain,
  overseerr: Tv,
  paperless: ScanSearch,
  plex: Film,
  prowlarr: Search,
  qbittorrent: HardDrive,
  qdrant: Database,
  radarr: Film,
  sabnzbd: HardDrive,
  sonarr: Tv,
  tailscale: Waypoints,
  tautulli: Film,
  tei: Bot,
  unifi: Wifi,
  unraid: Server,
}

function serviceIcon(service: SupportedService): LucideIcon {
  if (SERVICE_ICONS[service.key]) {
    return SERVICE_ICONS[service.key]
  }

  switch (service.category.toLowerCase()) {
    case 'ai':
      return Brain
    case 'documents':
      return BookMarked
    case 'download':
      return HardDrive
    case 'network':
      return Router
    case 'notifications':
      return Bell
    case 'notes':
      return Library
    case 'media':
      return Film
    default:
      return ShieldEllipsis
  }
}

export function LabServicePicker({
  selectedService,
  services,
  onSelect,
}: LabServicePickerProps) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">Supported services</p>
          <p className="text-sm text-muted-foreground">
            Configure any supported Lab service as its own virtual gateway server.
          </p>
        </div>
        <Badge variant="secondary">{services.length} services</Badge>
      </div>

      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
        {services.map((service) => {
          const Icon = serviceIcon(service)
          const active = selectedService === service.key

          return (
            <button
              key={service.key}
              type="button"
              onClick={() => onSelect(service.key)}
              aria-pressed={active}
              className={cn(
                'group rounded-xl border p-4 text-left transition',
                active
                  ? 'border-primary bg-primary/10 shadow-sm shadow-primary/10'
                  : 'border-border bg-card hover:border-primary/40 hover:bg-muted/40',
              )}
            >
              <div className="flex items-center gap-3">
                <div
                  className={cn(
                    'flex size-10 items-center justify-center rounded-lg border',
                    active
                      ? 'border-primary/30 bg-primary/15 text-primary'
                      : 'border-border bg-background text-muted-foreground group-hover:text-foreground',
                  )}
                >
                  <Icon className="size-5" />
                </div>
                <div className="min-w-0">
                  <p className="truncate font-medium">{service.display_name}</p>
                  <p className="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                    {service.category}
                  </p>
                </div>
              </div>

              <p className="mt-3 line-clamp-2 text-xs text-muted-foreground">
                {service.description}
              </p>
            </button>
          )
        })}
      </div>
    </div>
  )
}
