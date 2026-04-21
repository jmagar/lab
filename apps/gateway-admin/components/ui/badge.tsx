import * as React from 'react'
import { Slot } from '@radix-ui/react-slot'
import { cva, type VariantProps } from 'class-variance-authority'

import { cn } from '@/lib/utils'

const badgeVariants = cva(
  'inline-flex items-center justify-center rounded-md border px-2 py-0.5 text-xs font-medium w-fit whitespace-nowrap shrink-0 [&>svg]:size-3 gap-1 [&>svg]:pointer-events-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive transition-[color,box-shadow] overflow-hidden',
  {
    variants: {
      variant: {
        default:
          'border-transparent bg-primary text-primary-foreground [a&]:hover:bg-primary/90',
        secondary:
          'border-transparent bg-secondary text-secondary-foreground [a&]:hover:bg-secondary/90',
        /**
         * @deprecated Use `status="error"` instead. Retained one cycle for backwards compatibility.
         */
        destructive:
          'border-transparent bg-destructive text-white [a&]:hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60',
        outline:
          'text-foreground [a&]:hover:bg-accent [a&]:hover:text-accent-foreground',
        pill: 'border-transparent bg-primary text-primary-foreground rounded-full px-2.5 [a&]:hover:bg-primary/90',
      },
      status: {
        default: '',
        warn: '',
        error: '',
        success: '',
      },
    },
    compoundVariants: [
      // warn tones — applied to default, outline, pill (not secondary/destructive)
      {
        variant: 'default',
        status: 'warn',
        className: 'bg-aurora-warn/12 border-aurora-warn/30 text-aurora-warn',
      },
      {
        variant: 'outline',
        status: 'warn',
        className: 'bg-aurora-warn/12 border-aurora-warn/30 text-aurora-warn',
      },
      {
        variant: 'pill',
        status: 'warn',
        className: 'bg-aurora-warn/12 border-aurora-warn/30 text-aurora-warn',
      },
      // error tones
      {
        variant: 'default',
        status: 'error',
        className: 'bg-aurora-error/12 border-aurora-error/30 text-aurora-error',
      },
      {
        variant: 'outline',
        status: 'error',
        className: 'bg-aurora-error/12 border-aurora-error/30 text-aurora-error',
      },
      {
        variant: 'pill',
        status: 'error',
        className: 'bg-aurora-error/12 border-aurora-error/30 text-aurora-error',
      },
      // success tones
      {
        variant: 'default',
        status: 'success',
        className:
          'bg-aurora-success/12 border-aurora-success/30 text-aurora-success',
      },
      {
        variant: 'outline',
        status: 'success',
        className:
          'bg-aurora-success/12 border-aurora-success/30 text-aurora-success',
      },
      {
        variant: 'pill',
        status: 'success',
        className:
          'bg-aurora-success/12 border-aurora-success/30 text-aurora-success',
      },
    ],
    defaultVariants: {
      variant: 'default',
      status: 'default',
    },
  },
)

function Badge({
  className,
  variant,
  status,
  asChild = false,
  ...props
}: React.ComponentProps<'span'> &
  VariantProps<typeof badgeVariants> & { asChild?: boolean }) {
  const Comp = asChild ? Slot : 'span'

  return (
    <Comp
      data-slot="badge"
      className={cn(badgeVariants({ variant, status }), className)}
      {...props}
    />
  )
}

export { Badge, badgeVariants }
