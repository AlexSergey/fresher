'use strict';

// Only the staged files are checked (docs/quality.md, "Layer 3"). A function entry ignores the file list: Clippy and
// tsc are whole-project checks, run once per commit, and only when a matching file is staged. A `*.generated.ts` file
// is written by its generator byte for byte, so no formatter touches it.
module.exports = {
  '!(*.generated).{ts,tsx}': ['prettier --write', 'eslint --fix', () => 'npm run lint:ts'],
  '*.generated.ts': [() => 'npm run lint:ts'],
  '*.json': ['prettier --write', 'eslint --fix'],
  '*.rs': ['rustfmt --edition 2024', () => 'cargo clippy --workspace --all-targets --locked -- -D warnings'],
  '*.toml': ['taplo fmt'],
  '*.{js,cjs,mjs}': ['prettier --write', 'eslint --fix'],
  '*.{md,yml,yaml,css,html}': ['prettier --write'],
  '{Cargo.toml,Cargo.lock,deny.toml}': [() => ['cargo deny check', 'cargo machete']],
};
