import type { Linter } from 'eslint';

import eslintReact from '@eslint-react/eslint-plugin';
import js from '@eslint/js';
import json from '@eslint/json';
import vitest from '@vitest/eslint-plugin';
import gitignore from 'eslint-config-flat-gitignore';
import checkFile from 'eslint-plugin-check-file';
import importLite from 'eslint-plugin-import-lite';
import jsxA11y from 'eslint-plugin-jsx-a11y-x';
import noOnlyTests from 'eslint-plugin-no-only-tests';
import packageJsonConfig from 'eslint-plugin-package-json';
import perfectionist from 'eslint-plugin-perfectionist';
import playwright from 'eslint-plugin-playwright';
import prettierRecommended from 'eslint-plugin-prettier/recommended';
import regexpPlugin from 'eslint-plugin-regexp';
import sonar from 'eslint-plugin-sonarjs';
import testingLibrary from 'eslint-plugin-testing-library';
import unicorn from 'eslint-plugin-unicorn';
import globals from 'globals';
import tseslint from 'typescript-eslint';

// The rules mirror prod-forge/backend's eslint.config.ts (itself @rockpack/codestyle), adapted for React and Vitest;
// the differences are listed in docs/quality.md, "Layer 2".

const rootDir = import.meta.dirname;

const jsFiles = ['**/*.{js,jsx,mjs,cjs}'];
const tsFiles = ['**/*.{ts,tsx,mts,cts}'];
const sourceFiles = ['**/*.{js,jsx,mjs,cjs,ts,tsx,mts,cts}'];
const uiFiles = ['app/ui/src/**/*.{ts,tsx}'];
const unitTestFiles = ['app/ui/**/*.{spec,test}.{ts,tsx}'];
const e2eFiles = ['app/ui/e2e/**/*.ts'];
const configFiles = ['*.config.ts', 'app/ui/*.config.ts', '*.d.ts'];

const recommendedTypescriptConfigs = [
  ...tseslint.configs.strictTypeChecked.map((config) => ({ ...config, files: tsFiles })),
  ...tseslint.configs.stylisticTypeChecked.map((config) => ({ ...config, files: tsFiles })),
] as Linter.Config[];

const perfectionistConfig: Linter.Config = {
  files: sourceFiles,
  ...perfectionist.configs['recommended-natural'],
};

const regexpConfig: Linter.Config = {
  files: sourceFiles,
  ...regexpPlugin.configs['flat/recommended'],
};

const typescriptConfig: Linter.Config = {
  files: tsFiles,
  languageOptions: {
    ecmaVersion: 2024,
    parser: tseslint.parser,
    parserOptions: {
      projectService: true,
      tsconfigRootDir: rootDir,
    },
    sourceType: 'module',
  },
  plugins: {
    '@check-file': checkFile,
    '@import-lite': importLite,
    '@no-only-tests': noOnlyTests,
    '@sonar': sonar,
    '@typescript-eslint': tseslint.plugin,
    '@unicorn': unicorn,
  },
  rules: {
    '@check-file/filename-naming-convention': [
      'error',
      { 'app/ui/src/**/*.{ts,tsx}': 'KEBAB_CASE', 'scripts/**/*.ts': 'KEBAB_CASE' },
      { ignoreMiddleExtensions: true },
    ],
    '@check-file/folder-naming-convention': ['error', { 'app/ui/src/**/': 'KEBAB_CASE', 'scripts/**/': 'KEBAB_CASE' }],

    '@import-lite/no-default-export': 'error',

    '@no-only-tests/no-only-tests': 'error',

    '@sonar/cognitive-complexity': ['error', 20],
    '@sonar/no-all-duplicated-branches': 'error',
    '@sonar/no-collapsible-if': 'error',
    '@sonar/no-collection-size-mischeck': 'error',
    '@sonar/no-duplicated-branches': 'error',
    '@sonar/no-element-overwrite': 'error',
    '@sonar/no-empty-collection': 'error',
    '@sonar/no-gratuitous-expressions': 'error',
    '@sonar/no-identical-conditions': 'error',
    '@sonar/no-identical-expressions': 'error',
    '@sonar/no-identical-functions': 'error',
    '@sonar/no-ignored-return': 'error',
    '@sonar/no-inverted-boolean-check': 'error',
    '@sonar/no-redundant-boolean': 'error',
    '@sonar/no-small-switch': 'error',
    '@sonar/no-unthrown-error': 'error',
    '@sonar/no-unused-collection': 'error',
    '@sonar/no-use-of-empty-return-value': 'error',
    '@sonar/non-existent-operator': 'error',
    '@sonar/prefer-immediate-return': 'error',
    '@sonar/reduce-initial-value': 'error',

    '@typescript-eslint/ban-ts-comment': 'error',
    '@typescript-eslint/consistent-type-definitions': ['error', 'type'],
    '@typescript-eslint/consistent-type-imports': 'error',
    '@typescript-eslint/explicit-function-return-type': 'error',
    '@typescript-eslint/naming-convention': [
      'error',
      { format: ['PascalCase'], selector: 'typeLike' },
      { format: ['UPPER_CASE', 'StrictPascalCase'], selector: 'class' },
    ],
    '@typescript-eslint/no-confusing-void-expression': ['error', { ignoreArrowShorthand: true }],
    '@typescript-eslint/no-shadow': 'error',
    '@typescript-eslint/no-unused-vars': ['error', { args: 'after-used', ignoreRestSiblings: false, vars: 'all' }],
    // Conflicts with no-non-null-assertion from the strict preset: keep explicit `as` casts.
    '@typescript-eslint/non-nullable-type-assertion-style': 'off',
    '@typescript-eslint/prefer-readonly': 'error',
    '@typescript-eslint/restrict-template-expressions': ['error', { allowNumber: true }],
    '@typescript-eslint/return-await': 'off',
    '@typescript-eslint/switch-exhaustiveness-check': 'error',

    '@unicorn/error-message': 'error',
    '@unicorn/no-await-in-promise-methods': 'error',
    '@unicorn/no-instanceof-builtins': 'error',
    '@unicorn/no-single-promise-in-promise-methods': 'error',
    '@unicorn/no-thenable': 'error',
    '@unicorn/no-useless-promise-resolve-reject': 'error',
    '@unicorn/no-useless-spread': 'error',
    '@unicorn/no-useless-undefined': ['error', { checkArguments: false, checkArrowFunctionBody: false }],
    '@unicorn/prefer-array-flat': 'error',
    '@unicorn/prefer-modern-dom-apis': 'error',
    '@unicorn/prefer-node-protocol': 'error',
    '@unicorn/prefer-number-properties': 'error',
    '@unicorn/prefer-string-starts-ends-with': 'error',
    '@unicorn/prefer-structured-clone': 'error',
    '@unicorn/prefer-type-error': 'error',
    '@unicorn/throw-new-error': 'error',

    'array-callback-return': ['error', { allowImplicit: true }],
    camelcase: ['error', { properties: 'always' }],
    // `== null` still covers both null and undefined.
    eqeqeq: ['error', 'always', { null: 'ignore' }],
    'getter-return': ['error', { allowImplicit: true }],
    'newline-before-return': 'error',
    'no-alert': 'error',
    'no-console': 'error',
    'no-debugger': 'error',
    'no-unused-vars': 'off',
    'no-warning-comments': 'warn',
  },
};

// The UI: browser globals, React (its preset includes the rules of hooks) and accessibility.
const uiConfigs: Linter.Config[] = [
  {
    files: uiFiles,
    languageOptions: { globals: { ...globals.browser } },
  },
  {
    ...eslintReact.configs['recommended-type-checked'],
    files: uiFiles,
  },
  {
    ...(jsxA11y.configs.recommended as Linter.Config),
    files: uiFiles,
  },
];

// The UI's import boundaries (docs/plans/003-base-ui/00-design.md, "Folders and boundaries"). Each folder gets one
// config with its whole list, because a later config with the same rule replaces an earlier one.
type RestrictedPattern = { readonly allowTypeImports?: boolean; readonly message: string; readonly regex: string };

// A relative import that climbs out of its folder into one of `folders` of `src/`.
const climbingInto = (folders: string): string => String.raw`^(?:\.\./)+(?:${folders})(?:/|$)`;

const tauriApi: RestrictedPattern = {
  message: 'Only src/ipc/ talks to the Tauri API.',
  regex: '^@tauri-apps/api(?:/|$)',
};
const lucide: RestrictedPattern = {
  message: 'Icons come from design/icons/, the allowlist of the icons the app uses.',
  regex: '^lucide-react(?:/|$)',
};
const outOfDesign: RestrictedPattern = {
  message: 'design/ knows nothing of the product: no product, screens, state, ipc or i18n.',
  regex: climbingInto('product|screens|state|ipc|i18n'),
};

const boundary = (files: string[], ignores: string[], patterns: readonly RestrictedPattern[]): Linter.Config => ({
  files,
  ignores,
  rules: { '@typescript-eslint/no-restricted-imports': ['error', { patterns }] },
});

const boundaryConfigs: Linter.Config[] = [
  boundary(
    uiFiles,
    [
      'app/ui/src/ipc/**',
      // The unit tests' setup clears the Tauri mocks after each test.
      'app/ui/src/test/setup.ts',
      'app/ui/src/design/**',
      'app/ui/src/product/**',
      'app/ui/src/gallery/**',
    ],
    [tauriApi, lucide],
  ),
  boundary(['app/ui/src/design/**/*.{ts,tsx}'], ['app/ui/src/design/icons/**'], [tauriApi, lucide, outOfDesign]),
  boundary(['app/ui/src/design/icons/**/*.{ts,tsx}'], [], [tauriApi, outOfDesign]),
  boundary(
    ['app/ui/src/product/**/*.{ts,tsx}'],
    [],
    [
      tauriApi,
      lucide,
      { message: 'product/ takes its data by props: no state or screens.', regex: climbingInto('state|screens') },
      {
        message: 'product/ never calls the core: only types from ipc/bindings.generated.',
        regex: String.raw`^(?:\.\./)+ipc(?:/(?!bindings\.generated$).*)?$`,
      },
      {
        allowTypeImports: true,
        message: 'product/ takes only types from ipc/bindings.generated (import type).',
        regex: String.raw`^(?:\.\./)+ipc/bindings\.generated$`,
      },
    ],
  ),
  boundary(
    ['app/ui/src/gallery/**/*.{ts,tsx}'],
    [],
    [
      tauriApi,
      lucide,
      { message: 'The gallery renders components, not the app: no state.', regex: climbingInto('state') },
      { message: 'The gallery never calls the core.', regex: String.raw`^(?:\.\./)+ipc(?:/index)?$` },
    ],
  ),
];

// Tooling: configs and repository scripts run on Node.
const nodeConfig: Linter.Config = {
  files: [...configFiles, 'scripts/**/*.ts', ...jsFiles],
  languageOptions: { globals: { ...globals.node } },
};

const jsonCustomConfig: Linter.Config = {
  ...json.configs.recommended,
  files: ['**/*.json'],
  ignores: ['**/*-lock.json', '**/package.json', '**/tsconfig*.json'],
  language: 'json/json',
};

// tsconfig files allow comments and trailing commas.
const jsoncConfig: Linter.Config = {
  ...json.configs.recommended,
  files: ['**/tsconfig*.json'],
  language: 'json/jsonc',
};

const customPackageJsonConfig: Linter.Config = {
  files: ['**/package.json'],
  rules: {
    'package-json/require-exports': 'off',
    'package-json/require-files': 'off',
    'package-json/require-repository': 'off',
    'package-json/require-sideEffects': 'off',
  },
};

const customJsConfig: Linter.Config = {
  files: jsFiles,
  ...js.configs.recommended,
};

const defaultExportAllowed: Linter.Config = {
  files: [...configFiles, '**/*.d.ts'],
  rules: {
    '@import-lite/no-default-export': 'off',
  },
};

const unitTestOverrides: Linter.Config[] = [
  {
    files: unitTestFiles,
    plugins: { vitest },
    rules: {
      ...vitest.configs.recommended.rules,
      '@typescript-eslint/no-empty-function': 'off',
      '@typescript-eslint/unbound-method': 'off',
      'vitest/no-alias-methods': 'error',
      'vitest/no-conditional-expect': 'error',
      'vitest/no-disabled-tests': 'error',
      'vitest/no-focused-tests': 'error',
      'vitest/no-identical-title': 'error',
      'vitest/no-interpolation-in-snapshots': 'error',
      'vitest/no-mocks-import': 'error',
      'vitest/no-standalone-expect': 'error',
      'vitest/no-test-prefixes': 'error',
      'vitest/prefer-to-have-length': 'error',
      'vitest/valid-describe-callback': 'error',
      'vitest/valid-expect': 'error',
      'vitest/valid-title': 'error',
    },
  },
  {
    ...testingLibrary.configs['flat/react'],
    files: ['app/ui/src/**/*.test.tsx'],
  },
];

// The repository scripts are tested with node:test, whose describe() and it() return promises the runner awaits.
const scriptTestOverrides: Linter.Config = {
  files: ['scripts/**/*.test.ts'],
  rules: {
    '@typescript-eslint/no-floating-promises': 'off',
  },
};

const e2eOverrides: Linter.Config = {
  ...playwright.configs['flat/recommended'],
  files: e2eFiles,
};

export default [
  gitignore({ strict: false }),
  { ignores: ['app/tauri/gen/**', 'fixtures/**', '**/*.generated.ts'] },
  ...recommendedTypescriptConfigs,
  prettierRecommended,
  perfectionistConfig,
  regexpConfig,
  typescriptConfig,
  ...uiConfigs,
  ...boundaryConfigs,
  nodeConfig,
  jsonCustomConfig,
  jsoncConfig,
  packageJsonConfig.configs.recommended,
  customPackageJsonConfig,
  packageJsonConfig.configs.stylistic,
  customJsConfig,
  defaultExportAllowed,
  ...unitTestOverrides,
  scriptTestOverrides,
  e2eOverrides,
] satisfies Linter.Config[];
