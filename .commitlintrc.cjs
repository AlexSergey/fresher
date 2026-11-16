'use strict';

// Conventional Commits: `type(scope): subject` (docs/quality.md, "Layer 4").
module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'scope-enum': [
      2,
      'always',
      [
        'app',
        'ci',
        'cleaner',
        'core',
        'deps',
        'docs',
        'e2e',
        'engine',
        'macos',
        'modules',
        'repo',
        'rules',
        'scanner',
        'store',
        'system',
        'testing',
        'ui',
      ],
    ],
    'type-enum': [2, 'always', ['chore', 'ci', 'docs', 'feat', 'fix', 'perf', 'refactor', 'revert', 'style', 'test']],
  },
};
