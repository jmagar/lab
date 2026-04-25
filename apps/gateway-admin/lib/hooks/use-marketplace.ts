'use client'

import { useCallback } from 'react'
import useSWR from 'swr'
import { toast } from 'sonner'
import type { Marketplace, Plugin } from '../types/marketplace.js'
import {
  fetchMarketplaces,
  fetchPlugins,
  installPlugin,
  uninstallPlugin,
  addMarketplace,
} from '../api/marketplace-client'
import { listAcpAgents, listMcpServers } from '../marketplace/api-client'
import { MOCK_ACP_AGENTS } from '../marketplace/mocks'
import type { AcpAgent, McpServer } from '../marketplace/types'

const MARKETPLACES_KEY = 'marketplace:sources'
const PLUGINS_KEY = 'marketplace:plugins'
const ACP_AGENTS_KEY = 'marketplace:acp-agents'
const MCP_SERVERS_KEY = 'marketplace:mcp-servers'

export function useMarketplaces() {
  return useSWR<Marketplace[]>(MARKETPLACES_KEY, () => fetchMarketplaces(), {
    revalidateOnFocus: false,
    fallbackData: [],
  })
}

export function usePlugins() {
  return useSWR<Plugin[]>(PLUGINS_KEY, () => fetchPlugins(), {
    revalidateOnFocus: false,
    fallbackData: [],
  })
}

export function useAcpAgents() {
  return useSWR<AcpAgent[]>(ACP_AGENTS_KEY, () => listAcpAgents(), {
    revalidateOnFocus: false,
    fallbackData: MOCK_ACP_AGENTS,
  })
}

export function useMcpServers() {
  return useSWR<McpServer[]>(MCP_SERVERS_KEY, () => listMcpServers(), {
    revalidateOnFocus: false,
    fallbackData: [],
  })
}

export function useMarketplaceMutations() {
  const { mutate: mutatePlugins } = useSWR<Plugin[]>(PLUGINS_KEY)
  const { mutate: mutateMarketplaces } = useSWR<Marketplace[]>(MARKETPLACES_KEY)

  const install = useCallback(async (pluginId: string, pluginName: string) => {
    try {
      await installPlugin(pluginId)
      await mutatePlugins(async (prev = []) =>
        prev.map(p => p.id === pluginId ? { ...p, installed: true, hasUpdate: false, installedAt: new Date().toISOString() } : p)
      , { revalidate: false })
      toast.success(`Installed ${pluginName}`)
    } catch {
      toast.error(`Failed to install ${pluginName}`)
    }
  }, [mutatePlugins])

  const uninstall = useCallback(async (pluginId: string, pluginName: string) => {
    try {
      await uninstallPlugin(pluginId)
      await mutatePlugins(async (prev = []) =>
        prev.map(p => p.id === pluginId ? { ...p, installed: false, hasUpdate: false, installedAt: undefined } : p)
      , { revalidate: false })
      toast.success(`Removed ${pluginName}`)
    } catch {
      toast.error(`Failed to remove ${pluginName}`)
    }
  }, [mutatePlugins])

  const addSource = useCallback(async (input: Parameters<typeof addMarketplace>[0]) => {
    try {
      const mkt = await addMarketplace(input)
      await mutateMarketplaces(async (prev = []) => [...prev, mkt], { revalidate: false })
      await mutatePlugins(() => fetchPlugins(), { revalidate: false })
      toast.success(`Added ${mkt.name}`)
      return mkt
    } catch {
      toast.error('Failed to add marketplace')
      return null
    }
  }, [mutateMarketplaces])

  return { install, uninstall, addSource }
}
