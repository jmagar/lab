import { Compartment, type Extension } from '@codemirror/state'
import { EditorView, drawSelection, highlightActiveLine, lineNumbers, keymap } from '@codemirror/view'
import { defaultHighlightStyle, syntaxHighlighting, foldGutter } from '@codemirror/language'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { search, searchKeymap } from '@codemirror/search'
import { autocompletion, closeBrackets, completionKeymap } from '@codemirror/autocomplete'
import { lintGutter, linter } from '@codemirror/lint'

import type { EditorDiagnostic } from '@/lib/editor/types'

export const languageCompartment = new Compartment()
export const editableCompartment = new Compartment()
export const diagnosticsCompartment = new Compartment()

function severityClass(severity: EditorDiagnostic['severity']): 'error' | 'warning' | 'info' {
  return severity === 'error' ? 'error' : severity === 'warning' ? 'warning' : 'info'
}

export function auroraTextSurfaceTheme(): Extension {
  return [
    EditorView.theme({
      '&': {
        color: 'var(--aurora-text-primary)',
        backgroundColor: 'transparent',
        fontSize: '13px',
      },
      '.cm-content': {
        fontFamily: 'var(--font-geist-mono, ui-monospace, SFMono-Regular, monospace)',
        caretColor: 'var(--aurora-text-primary)',
      },
      '.cm-cursor, .cm-dropCursor': { borderLeftColor: 'var(--aurora-text-primary)' },
      '&.cm-focused': { outline: 'none' },
      '.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'color-mix(in srgb, var(--aurora-accent-primary) 8%, transparent)' },
      '.cm-selectionBackground, ::selection': { backgroundColor: 'color-mix(in srgb, var(--aurora-accent-primary) 22%, transparent)' },
      '.cm-gutters': {
        backgroundColor: 'var(--aurora-control-surface)',
        color: 'var(--aurora-text-muted)',
        borderRight: '1px solid var(--aurora-border-default)',
      },
      '.cm-panels, .cm-tooltip': {
        backgroundColor: 'var(--aurora-panel-strong)',
        color: 'var(--aurora-text-primary)',
        border: '1px solid var(--aurora-border-strong)',
      },
    }),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
  ]
}

export function diagnosticsExtension(diagnostics: EditorDiagnostic[]): Extension {
  return linter(() => diagnostics.map((item) => ({
    from: item.from,
    to: item.to,
    severity: severityClass(item.severity),
    message: item.message,
  })))
}

export function baseTextSurfaceExtensions({ editable, diagnostics }: { editable: boolean; diagnostics: EditorDiagnostic[] }): Extension[] {
  return [
    lineNumbers(),
    highlightActiveLine(),
    drawSelection(),
    history(),
    foldGutter(),
    search({ top: true }),
    autocompletion(),
    closeBrackets(),
    lintGutter(),
    keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, ...completionKeymap]),
    languageCompartment.of([]),
    editableCompartment.of(EditorView.editable.of(editable)),
    diagnosticsCompartment.of(diagnosticsExtension(diagnostics)),
    auroraTextSurfaceTheme(),
  ]
}
