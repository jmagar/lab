export const auroraColorTokens = [
  {
    label: 'Aurora page background',
    value: '#07131c',
    className: 'bg-aurora-page-bg',
    textClassName: 'text-aurora-text-primary',
  },
  {
    label: 'Aurora panel medium',
    value: '#102330',
    className: 'bg-aurora-panel-medium',
    textClassName: 'text-aurora-text-primary',
  },
  {
    label: 'Aurora control surface',
    value: '#0c1a24',
    className: 'bg-aurora-control-surface',
    textClassName: 'text-aurora-text-primary',
  },
  {
    label: 'Aurora accent',
    value: '#29b6f6',
    className: 'bg-aurora-accent-primary',
    textClassName: 'text-[#06253a]',
  },
  {
    label: 'Aurora warn',
    value: '#c6a36b',
    className: 'bg-aurora-warn',
    textClassName: 'text-[#2f2110]',
  },
  {
    label: 'Aurora error',
    value: '#c78490',
    className: 'bg-aurora-error',
    textClassName: 'text-[#311319]',
  },
] as const

export const typeRamp = [
  {
    label: 'Display 1',
    className: 'font-display text-4xl font-semibold tracking-[-0.05em] text-aurora-text-primary',
    preview: '72',
  },
  {
    label: 'Display 2',
    className: 'font-display text-3xl font-semibold tracking-[-0.04em] text-aurora-text-primary',
    preview: 'Gateway Fleet',
  },
  {
    label: 'Metric Display',
    className: 'font-display text-2xl font-semibold tracking-[-0.05em] text-aurora-accent-strong tabular-nums',
    preview: '189',
  },
  {
    label: 'Body',
    className: 'text-base text-aurora-text-primary',
    preview: 'Default interface copy tuned for readable admin panels.',
  },
  {
    label: 'Control',
    className: 'text-sm font-medium text-aurora-text-primary',
    preview: 'Primary action',
  },
  {
    label: 'Dense Data',
    className: 'font-mono text-xs leading-6 text-aurora-text-muted',
    preview: 'gateway.request.start',
  },
  {
    label: 'Eyebrow',
    className: 'text-[11px] font-medium uppercase tracking-[0.18em] text-aurora-text-muted',
    preview: 'Sandbox heading',
  },
] as const

export const spacingScale = [
  { label: 'Space 2', value: '0.5rem' },
  { label: 'Space 4', value: '1rem' },
  { label: 'Space 6', value: '1.5rem' },
  { label: 'Space 8', value: '2rem' },
] as const

export const radiusScale = [
  { label: 'Radius 1', className: 'rounded-md', value: '0.625rem' },
  { label: 'Radius 2', className: 'rounded-xl', value: '0.875rem' },
  { label: 'Radius 3', className: 'rounded-[1.4rem]', value: '1.4rem' },
] as const

export const elevationTiers = [
  {
    label: 'Tier 1 lift',
    description: 'Default panels and cards with medium emphasis.',
    className: 'shadow-[var(--aurora-shadow-medium),var(--aurora-highlight-medium)]',
  },
  {
    label: 'Tier 2 lift',
    description: 'Pinned inspectors and hero surfaces with stronger separation.',
    className: 'shadow-[var(--aurora-shadow-strong),var(--aurora-highlight-strong)]',
  },
] as const

export const environmentOptions = [
  { value: 'preview', label: 'Preview cluster' },
  { value: 'prod', label: 'Production' },
  { value: 'edge', label: 'Edge canary' },
] as const

export const pillFilterOptions = [
  'Healthy only',
  'Warnings',
  'Protected routes',
  'Paused rollouts',
] as const

export const accessModeOptions = [
  { value: 'session', label: 'Session auth' },
  { value: 'token', label: 'Bearer token' },
  { value: 'mock', label: 'Mock data' },
] as const

export const feedbackModes = [
  'loading',
  'empty',
  'success',
  'warning',
  'error',
] as const
