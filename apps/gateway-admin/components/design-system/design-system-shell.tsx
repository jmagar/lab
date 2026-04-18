import { AppHeader } from '@/components/app-header'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { ControlsSection } from './controls-section'
import { DataDisplaySection } from './data-display-section'
import { FeedbackSection } from './feedback-section'
import { FoundationsSection } from './foundations-section'
import { NavigationSection } from './navigation-section'

const sectionHeadings = [
  'Theme Foundations',
  'Controls',
  'Feedback',
  'Navigation',
  'Data Display',
  'Application Patterns',
] as const

export function DesignSystemShell() {
  return (
    <>
      <AppHeader
        breadcrumbs={[
          { label: 'Overview', href: '/' },
          { label: 'Design System' },
        ]}
      />
      <div className="flex-1 space-y-6 p-6">
        <Card className="overflow-hidden border-primary/15 bg-[radial-gradient(circle_at_top_left,rgba(0,176,255,0.14),transparent_45%),linear-gradient(180deg,rgba(255,255,255,0.98),rgba(245,249,252,0.96))]">
          <CardHeader className="border-b border-primary/10">
            <p className="text-xs font-medium uppercase tracking-[0.22em] text-primary/80">
              Internal Sandbox
            </p>
            <CardTitle className="text-3xl tracking-tight">Design System</CardTitle>
            <CardDescription className="max-w-2xl text-sm sm:text-base">
              A direct-url-only playground for validating layout, typography, controls, and
              application patterns against the current admin shell without calling real backend
              services.
            </CardDescription>
          </CardHeader>
        </Card>

        <div className="grid gap-4 xl:grid-cols-2">
          <div className="xl:col-span-2">
            <FoundationsSection />
          </div>
          <div className="xl:col-span-2">
            <ControlsSection />
          </div>
          <div className="xl:col-span-2">
            <FeedbackSection />
          </div>
          <div className="xl:col-span-2">
            <NavigationSection />
          </div>
          <div className="xl:col-span-2">
            <DataDisplaySection />
          </div>
          {sectionHeadings.filter((heading) => !['Theme Foundations', 'Controls', 'Feedback', 'Navigation', 'Data Display'].includes(heading)).map((heading) => (
            <Card key={heading} className="h-full">
              <CardHeader className="border-b">
                <CardTitle>{heading}</CardTitle>
                <CardDescription>
                  Section scaffold in place. Detailed interactive demos land here in later tasks.
                </CardDescription>
              </CardHeader>
              <CardContent className="pt-6 text-sm text-muted-foreground">
                Preview content intentionally stays local to this route so teams can test the UI
                safely.
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </>
  )
}
