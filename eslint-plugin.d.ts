// Plugins that ship no type declarations.
declare module 'eslint-plugin-no-only-tests' {
  import type { ESLint } from 'eslint';

  const plugin: ESLint.Plugin;
  export default plugin;
}
