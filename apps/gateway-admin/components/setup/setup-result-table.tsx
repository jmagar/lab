import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import type { ExtractCredential } from '@/lib/api/extract-client'

function SecretCell({ secretPresent }: { secretPresent: boolean }) {
  if (!secretPresent) {
    return <span className="text-aurora-text-muted">No secret</span>
  }

  return (
    <Badge variant="secondary" className="font-normal">
      Present
    </Badge>
  )
}

export function SetupResultTable({ creds }: { creds: ExtractCredential[] }) {
  return (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Service</TableHead>
          <TableHead>Host</TableHead>
          <TableHead>URL</TableHead>
          <TableHead>Env Field</TableHead>
          <TableHead>Secret</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {creds.map((cred) => (
          <TableRow key={`${cred.source_host ?? 'local'}:${cred.service}:${cred.env_field}`}>
            <TableCell className="font-medium capitalize">{cred.service}</TableCell>
            <TableCell>{cred.source_host ?? 'local'}</TableCell>
            <TableCell className="max-w-[20rem] truncate">
              {cred.url ? (
                <span title={cred.url}>{cred.url}</span>
              ) : (
                <span className="text-aurora-text-muted">No URL</span>
              )}
            </TableCell>
            <TableCell>
              <code className="rounded bg-aurora-control-surface px-2 py-1 text-xs">{cred.env_field}</code>
            </TableCell>
            <TableCell className="max-w-[18rem]">
              <SecretCell secretPresent={cred.secret_present} />
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  )
}
