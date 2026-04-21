/**
 * Shared gateway name validation.
 *
 * The Rust backend requires only that the name is non-empty.
 * This client-side rule is stricter to guide users toward names that are
 * portable across all surfaces (CLI flags, env var derivation, file paths):
 *   - starts with a letter or digit
 *   - contains only letters, digits, hyphens, or underscores
 *   - max 64 characters
 */
const GATEWAY_NAME_PATTERN = /^[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}$/

/**
 * Validate a gateway name.
 * Returns `null` when valid, or a human-readable error message when invalid.
 */
export function validateGatewayName(value: string): string | null {
  if (!value) return 'Gateway name is required'
  if (!GATEWAY_NAME_PATTERN.test(value))
    return 'Must start with a letter or digit and contain only letters, digits, underscores, or hyphens (max 64 chars)'
  return null
}
