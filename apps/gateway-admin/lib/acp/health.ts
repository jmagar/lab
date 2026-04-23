import { spawnSync } from 'node:child_process'
import type { ProviderHealth } from './types'

export function resolveCodexLaunch() {
  const configured = process.env.ACP_CODEX_COMMAND?.trim()
  const configuredArgs = (process.env.ACP_CODEX_ARGS ?? '')
    .split(' ')
    .map((value) => value.trim())
    .filter(Boolean)

  if (configured) {
    return { command: configured, args: configuredArgs }
  }

  const npxCommand = process.platform === 'win32' ? 'npx.cmd' : 'npx'
  return { command: npxCommand, args: ['@zed-industries/codex-acp'] }
}

export function getCodexProviderHealth(): ProviderHealth {
  const launch = resolveCodexLaunch()

  if (process.env.ACP_CODEX_COMMAND?.trim()) {
    return {
      provider: 'codex',
      ready: true,
      command: launch.command,
      args: launch.args,
      message: 'Using configured ACP Codex command.',
    }
  }

  const whichCommand = process.platform === 'win32' ? 'where' : 'which'
  const probe = spawnSync(whichCommand, [launch.command], { encoding: 'utf8' })

  return {
    provider: 'codex',
    ready: probe.status === 0,
    command: launch.command,
    args: launch.args,
    message:
      probe.status === 0
        ? 'Using npx to launch @zed-industries/codex-acp.'
        : 'npx was not found on PATH. Set ACP_CODEX_COMMAND or install Node tooling.',
  }
}
