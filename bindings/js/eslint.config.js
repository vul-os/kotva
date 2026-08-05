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
  {
    files: ['src/**/*.ts', 'test/**/*.ts'],
    extends: [
      js.configs.recommended,
      ...tseslint.configs.strictTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
])
