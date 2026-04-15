'use client'

import { HelpCircle, ExternalLink } from 'lucide-react'
import { AppHeader } from '@/components/app-header'
import { Button } from '@/components/ui/button'

export default function DocsPage() {
  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Documentation' }
        ]}
      />
      <div className="flex-1 p-6">
        <div className="rounded-lg border bg-card p-8">
          <div className="max-w-2xl mx-auto">
            <div className="flex size-12 items-center justify-center rounded-full bg-muted mb-6">
              <HelpCircle className="size-6 text-muted-foreground" />
            </div>
            
            <h1 className="text-2xl font-semibold mb-2">Labby Documentation</h1>
            <p className="text-muted-foreground mb-8">
              Learn how to configure and manage your MCP gateway connections.
            </p>

            <div className="space-y-4">
              <div className="rounded-lg border p-4">
                <h3 className="font-medium mb-1">What is MCP?</h3>
                <p className="text-sm text-muted-foreground">
                  Model Context Protocol (MCP) is an open protocol for connecting AI models to external tools and data sources.
                </p>
              </div>

              <div className="rounded-lg border p-4">
                <h3 className="font-medium mb-1">Gateway Concepts</h3>
                <p className="text-sm text-muted-foreground">
                  A gateway is an upstream MCP server connection managed by lab. Each gateway exposes MCP tools, resources, and prompts that can be republished downstream.
                </p>
              </div>

              <div className="rounded-lg border p-4">
                <h3 className="font-medium mb-1">Transport Types</h3>
                <p className="text-sm text-muted-foreground">
                  Gateways support two transport types: HTTP for network-based servers and stdio for local process-based servers.
                </p>
              </div>

              <div className="rounded-lg border p-4">
                <h3 className="font-medium mb-1">Exposure Policies</h3>
                <p className="text-sm text-muted-foreground">
                  Control which tools are exposed downstream using allowlist patterns. Use exact names like <code className="bg-muted px-1 rounded">read_file</code> or wildcards like <code className="bg-muted px-1 rounded">search_*</code>.
                </p>
              </div>
            </div>

            <div className="mt-8 pt-6 border-t">
              <Button variant="outline" asChild>
                <a href="https://modelcontextprotocol.io" target="_blank" rel="noopener noreferrer">
                  MCP Specification
                  <ExternalLink className="size-4 ml-2" />
                </a>
              </Button>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
