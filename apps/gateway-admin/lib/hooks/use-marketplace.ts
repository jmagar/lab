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

const MARKETPLACES_KEY = 'marketplace:sources'
const PLUGINS_KEY = 'marketplace:plugins'

export function useMarketplaces() {
  return useSWR<Marketplace[]>(MARKETPLACES_KEY, fetchMarketplaces, {
    revalidateOnFocus: false,
    fallbackData: [],
  })
}

export function usePlugins() {
  return useSWR<Plugin[]>(PLUGINS_KEY, fetchPlugins, {
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
        prev.map(p => p.id === pluginId ? { ...p, installed: true, installedAt: new Date().toISOString() } : p)
      )
      toast.success(`Installed ${pluginName}`)
    } catch {
      toast.error(`Failed to install ${pluginName}`)
    }
  }, [mutatePlugins])

  const uninstall = useCallback(async (pluginId: string, pluginName: string) => {
    try {
      await uninstallPlugin(pluginId)
      await mutatePlugins(async (prev = []) =>
        prev.map(p => p.id === pluginId ? { ...p, installed: false } : p)
      )
      toast.success(`Removed ${pluginName}`)
    } catch {
      toast.error(`Failed to remove ${pluginName}`)
    }
  }, [mutatePlugins])

  const addSource = useCallback(async (input: Parameters<typeof addMarketplace>[0]) => {
    try {
      const mkt = await addMarketplace(input)
      await mutateMarketplaces(async (prev = []) => [...prev, mkt])
      toast.success(`Added ${mkt.name}`)
      return mkt
    } catch {
      toast.error('Failed to add marketplace')
      return null
    }
  }, [mutateMarketplaces])

  return { install, uninstall, addSource }
}
