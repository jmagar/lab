function StatCard({
  label,
  value,
  description,
}: {
  label: string
  value: number
  description: string
}) {
  return (
    <div className="rounded-xl border bg-card/80 p-4 shadow-sm shadow-black/5">
      <p className="text-xs font-medium uppercase tracking-[0.2em] text-muted-foreground">{label}</p>
      <p className="mt-2 text-3xl font-semibold">{value}</p>
      <p className="mt-1 text-sm text-muted-foreground">{description}</p>
    </div>
  )
}

export function SetupStatGrid({
  summary,
}: {
  summary: {
    found: number
    creds: number
    warnings: number
  }
}) {
  return (
    <div className="grid gap-3 sm:grid-cols-3 xl:grid-cols-1">
      <StatCard
        label="Found"
        value={summary.found}
        description="Distinct supported services discovered in the latest scan."
      />
      <StatCard
        label="Credentials"
        value={summary.creds}
        description="Verified URL and credential entries returned by extract."
      />
      <StatCard
        label="Warnings"
        value={summary.warnings}
        description="Hosts or services that still need attention."
      />
    </div>
  )
}
