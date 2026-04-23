import { spawn } from 'node:child_process'
import { readFile, writeFile } from 'node:fs/promises'
import { Readable, Writable } from 'node:stream'
import {
  ClientSideConnection,
  PROTOCOL_VERSION,
  ndJsonStream,
  type Client,
  type PromptResponse,
  type RequestPermissionRequest,
  type RequestPermissionResponse,
  type SessionNotification,
} from '@agentclientprotocol/sdk'
import type { AcpProvider, AcpProviderHandle, ProviderEventHandler } from '../provider'
import { getCodexProviderHealth, resolveCodexLaunch } from '../health'
import type { ProviderHealth, StartSessionInput, StartSessionResult } from '../types'

type CodexHandle = AcpProviderHandle & {
  process: ReturnType<typeof spawn>
  connection: ClientSideConnection
}

type EventfulConnection = ClientSideConnection & {
  __onEvent?: ProviderEventHandler
}

function setEventHandler(connection: ClientSideConnection, onEvent: ProviderEventHandler) {
  ;(connection as EventfulConnection).__onEvent = onEvent
}

function emitConnectionEvent(connection: ClientSideConnection, event: Parameters<ProviderEventHandler>[0]) {
  ;(connection as EventfulConnection).__onEvent?.(event)
}

class CodexBridgeClient implements Client {
  constructor(
    private readonly onEvent: ProviderEventHandler,
  ) {}

  async requestPermission(params: RequestPermissionRequest): Promise<RequestPermissionResponse> {
    this.onEvent({ type: 'permission_request', request: params })

    const selected =
      params.options.find((option) => option.kind === 'allow_once') ??
      params.options.find((option) => option.kind === 'allow_always') ??
      params.options[0]

    const selectedOptionId = selected?.optionId ?? null

    this.onEvent({
      type: 'permission_resolved',
      request: params,
      selectedOptionId,
    })

    return {
      outcome: selectedOptionId
        ? {
            outcome: 'selected',
            optionId: selectedOptionId,
          }
        : {
            outcome: 'cancelled',
          },
    }
  }

  async sessionUpdate(params: SessionNotification): Promise<void> {
    this.onEvent({ type: 'session_notification', notification: params })
  }

  async readTextFile(params: { path: string }): Promise<{ content: string }> {
    const content = await readFile(params.path, 'utf8')
    return { content }
  }

  async writeTextFile(params: { path: string; content: string }): Promise<Record<string, never>> {
    await writeFile(params.path, params.content, 'utf8')
    return {}
  }
}

export class CodexAcpProvider implements AcpProvider {
  readonly kind = 'codex' as const
  private readonly handles = new Map<string, CodexHandle>()

  async health(): Promise<ProviderHealth> {
    return getCodexProviderHealth()
  }

  async startSession(input: StartSessionInput, onEvent: ProviderEventHandler): Promise<{
    handle: AcpProviderHandle
    result: StartSessionResult
  }> {
    const launch = resolveCodexLaunch()
    const child = spawn(launch.command, launch.args, {
      cwd: input.cwd,
      env: process.env,
      stdio: ['pipe', 'pipe', 'pipe'],
    })

    if (!child.stdin || !child.stdout) {
      throw new Error('Unable to open stdio streams for codex-acp')
    }

    child.stderr?.on('data', (chunk) => {
      const text = chunk.toString().trim()
      if (text) {
        onEvent({ type: 'stderr', text })
      }
    })

    child.on('exit', (code, signal) => {
      onEvent({ type: 'process_exit', code, signal })
    })

    const inputStream = Writable.toWeb(child.stdin) as WritableStream<Uint8Array<ArrayBufferLike>>
    const outputStream = Readable.toWeb(child.stdout) as ReadableStream<Uint8Array<ArrayBufferLike>>
    const stream = ndJsonStream(inputStream, outputStream)
    const client = new CodexBridgeClient(onEvent)
    const connection = new ClientSideConnection(() => client, stream)

    const init = await connection.initialize({
      protocolVersion: PROTOCOL_VERSION,
      clientInfo: {
        name: 'gateway-admin',
        title: 'Gateway Admin',
        version: '0.2.3',
      },
      clientCapabilities: input.clientCapabilities ?? {
        fs: {
          readTextFile: true,
          writeTextFile: true,
        },
      },
    })

    const created = await connection.newSession({
      cwd: input.cwd,
      mcpServers: [],
    })

    const handle: CodexHandle = {
      providerSessionId: created.sessionId,
      process: child,
      connection,
    }

    setEventHandler(connection, onEvent)

    this.handles.set(created.sessionId, handle)

    return {
      handle,
      result: {
        providerSessionId: created.sessionId,
        agentName: init.agentInfo?.title ?? init.agentInfo?.name ?? 'Codex ACP',
        agentVersion: init.agentInfo?.version ?? 'unknown',
        capabilities: init.agentCapabilities,
      },
    }
  }

  async promptSession(handle: AcpProviderHandle, prompt: string): Promise<void> {
    const codexHandle = this.handles.get(handle.providerSessionId)
    if (!codexHandle) {
      throw new Error(`Unknown ACP session: ${handle.providerSessionId}`)
    }

    void codexHandle.connection
      .prompt({
        sessionId: codexHandle.providerSessionId,
        messageId: crypto.randomUUID(),
        prompt: [{ type: 'text', text: prompt }],
      })
      .then((response: PromptResponse) => {
        emitConnectionEvent(codexHandle.connection, {
          type: 'prompt_response',
          response,
        })
      })
      .catch((error: unknown) => {
        emitConnectionEvent(codexHandle.connection, {
          type: 'error',
          message: error instanceof Error ? error.message : 'Prompt failed',
          raw: error,
        })
      })

    emitConnectionEvent(codexHandle.connection, {
      type: 'prompt_started',
      prompt,
    })
  }

  async cancelSession(handle: AcpProviderHandle): Promise<void> {
    const codexHandle = this.handles.get(handle.providerSessionId)
    if (!codexHandle) {
      return
    }
    await codexHandle.connection.cancel({ sessionId: codexHandle.providerSessionId })
  }

  async shutdownSession(handle: AcpProviderHandle): Promise<void> {
    const codexHandle = this.handles.get(handle.providerSessionId)
    if (!codexHandle) {
      return
    }

    try {
      await codexHandle.connection.cancel({ sessionId: codexHandle.providerSessionId })
    } catch {}

    codexHandle.process.kill()
    this.handles.delete(handle.providerSessionId)
  }

  async listSessions(): Promise<Array<{ providerSessionId: string }>> {
    return Array.from(this.handles.keys()).map((providerSessionId) => ({ providerSessionId }))
  }
}

export function attachProviderEventHandler(
  handle: AcpProviderHandle,
  onEvent: ProviderEventHandler,
): AcpProviderHandle {
  const codexHandle = handle as CodexHandle
  if (codexHandle.connection) {
    setEventHandler(codexHandle.connection, onEvent)
  }
  return handle
}
