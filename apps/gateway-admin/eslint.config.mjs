import js from '@eslint/js'
import globals from 'globals'
import tseslint from 'typescript-eslint'

// Banned shadcn-generic tokens in product code. Aurora tokens are the source of
// truth — see docs/design-system-contract.md. Primitives in components/ui/**
// are the sanctioned escape hatch and are exempted below.
const BANNED_TOKENS_PATTERN =
  String.raw`\b(text-muted-foreground|text-foreground|bg-card|bg-background|bg-muted|border-border)\b`

const bannedTokenRules = {
  'no-restricted-syntax': [
    'error',
    {
      selector: `JSXAttribute[name.name='className'] Literal[value=/${BANNED_TOKENS_PATTERN}/]`,
      message:
        'Use Aurora tokens instead of shadcn-generic classes (text-aurora-text-muted, bg-aurora-panel-medium, border-aurora-border-strong, etc). See docs/design-system-contract.md.',
    },
    {
      selector: `JSXAttribute[name.name='className'] TemplateElement[value.raw=/${BANNED_TOKENS_PATTERN}/]`,
      message:
        'Use Aurora tokens instead of shadcn-generic classes (text-aurora-text-muted, bg-aurora-panel-medium, border-aurora-border-strong, etc). See docs/design-system-contract.md.',
    },
    {
      selector: `VariableDeclarator[init.type='Literal'][init.value=/${BANNED_TOKENS_PATTERN}/]`,
      message:
        'Use Aurora tokens instead of shadcn-generic classes in shared class constants. See docs/design-system-contract.md.',
    },
    {
      selector: `VariableDeclarator[init.type='TemplateLiteral'] TemplateElement[value.raw=/${BANNED_TOKENS_PATTERN}/]`,
      message:
        'Use Aurora tokens instead of shadcn-generic classes in shared class constants. See docs/design-system-contract.md.',
    },
    {
      selector: `ArrayExpression > Literal[value=/${BANNED_TOKENS_PATTERN}/]`,
      message:
        'Use Aurora tokens instead of shadcn-generic classes inside clsx/cn arrays. See docs/design-system-contract.md.',
    },
    {
      selector: `ArrayExpression TemplateElement[value.raw=/${BANNED_TOKENS_PATTERN}/]`,
      message:
        'Use Aurora tokens instead of shadcn-generic classes inside clsx/cn arrays. See docs/design-system-contract.md.',
    },
  ],
}

export default tseslint.config(
  {
    ignores: [
      '.next/**',
      'node_modules/**',
      'out/**',
      'tsconfig.tsbuildinfo',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['**/*.{js,mjs,cjs}'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        ...globals.node,
      },
    },
  },
  {
    files: ['**/*.{ts,tsx,mts,cts}'],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        ...globals.browser,
        ...globals.node,
      },
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
    },
  },
  {
    files: ['app/**/*.{ts,tsx}', 'components/**/*.{ts,tsx}'],
    rules: bannedTokenRules,
  },
  {
    files: ['components/ui/**/*.{ts,tsx}'],
    rules: {
      'no-restricted-syntax': 'off',
    },
  },
)
