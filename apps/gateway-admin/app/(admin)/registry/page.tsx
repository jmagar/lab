import { AppHeader } from '@/components/app-header'

export default function RegistryPage() {
  return (
    <>
      <AppHeader breadcrumbs={[{ label: 'Registry' }]} />
      <div className="flex-1 p-6">
        <div className="flex h-32 items-center justify-center text-muted-foreground">
          Loading registry…
        </div>
      </div>
    </>
  )
}
