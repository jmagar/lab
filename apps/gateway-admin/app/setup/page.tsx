'use client'

// /setup root — the layout effect handles redirection to the right
// step. Render a skeleton so direct hits don't flash empty.

import { Loader2 } from 'lucide-react'

export default function SetupIndex(): React.ReactElement {
  return (
    <div className="flex items-center gap-2 text-sm text-muted-foreground">
      <Loader2 className="h-4 w-4 animate-spin" />
      Detecting setup state…
    </div>
  )
}
