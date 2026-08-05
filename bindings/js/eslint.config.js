import js from '@eslint/js'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

export default defineConfig([
  globalIgnores(['node_modules', 'coverage']),
  // kotva-client is protocol/crypto code that must agree byte-for-byte with a
  // Rust implementation validated by a vector corpus. Type-aware rules are
  // enabled (parserOptions.projectService) specifically so
  // no-floating-promises and the no-unsafe-* family actually run — a dropped
  // promise or an unchecked `any` crossing into a crypto call is exactly the
  // class of bug worth catching here.
  //
  // Level: recommendedTypeChecked, not strictTypeChecked. Measured both
  // (2026-08-05): strictTypeChecked = 294 errors, recommendedTypeChecked =
  // 135. The +159 delta is almost entirely no-non-null-assertion (106) plus
  // no-unnecessary-condition/restrict-template-expressions/
  // restrict-plus-operands/no-unnecessary-type-conversion/
  // no-confusing-void-expression overreach fighting deliberate idioms in
  // this codebase — the same pattern strictTypeChecked produced in every
  // other fleet repo except llmux (+7). recommendedTypeChecked keeps every
  // rule that matters for this file's job (no-floating-promises,
  // no-misused-promises, the full no-unsafe-* family) and aligns kotva with
  // the other twelve repos' measured decision.
  {
    files: ['src/**/*.ts', 'test/**/*.ts'],
    extends: [
      js.configs.recommended,
      ...tseslint.configs.recommendedTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
])
