import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, describe, it } from 'node:test';

import type { CargoAboutLicence } from './notices-build.ts';

import { NOTICES_PATH, run } from './notices.ts';

const DENY = `[licenses]
allow = ["ISC", "MIT"]
exceptions = [{ crate = "selectors", allow = ["MPL-2.0"] }]

[licenses.private]
ignore = true
`;

const ABOUT = `accepted = ["ISC", "MIT"]

[selectors]
accepted = ["MPL-2.0"]
`;

// One licence of cargo-about's report, under its own (snake case) key.
const licence = (
  id: string,
  name: string,
  text: string,
  crates: readonly (readonly [string, string])[],
): CargoAboutLicence => ({
  id,
  name,
  text,
  ['used_by']: crates.map(([crateName, version]) => ({ crate: { name: crateName, version } })),
});

const REPORT = JSON.stringify({
  licenses: [
    licence('MIT', 'MIT License', 'MIT text', [['serde', '1.0.0']]),
    licence('MPL-2.0', 'Mozilla Public License 2.0', 'MPL text', [['selectors', '0.1.0']]),
  ],
});

const LOCK = {
  packages: {
    '': { devDependencies: { tool: '1.0.0' } },
    'app/ui': { version: '0.1.0' },
    'node_modules/@fresher/ui': { link: true },
    'node_modules/react': { license: 'MIT', version: '19.0.0' },
    'node_modules/tool': { dev: true, license: 'GPL-3.0-only', version: '1.0.0' },
  },
};

const root = mkdtempSync(path.join(tmpdir(), 'notices-'));
let count = 0;

after(() => {
  rmSync(root, { force: true, recursive: true });
});

const write = (folder: string, file: string, text: string): void => {
  mkdirSync(path.dirname(path.join(folder, file)), { recursive: true });
  writeFileSync(path.join(folder, file), text);
};

// A repository with the given react package: its manifest and its licence file.
const repo = (react: { readonly licence?: string; readonly version?: string } = {}): string => {
  count += 1;
  const folder = path.join(root, `repo-${String(count)}`);
  write(folder, 'deny.toml', DENY);
  write(folder, 'about.toml', ABOUT);
  write(folder, 'package-lock.json', JSON.stringify(LOCK));
  write(folder, '.prettierrc', JSON.stringify({ printWidth: 120 }));
  write(
    folder,
    'node_modules/react/package.json',
    JSON.stringify({ license: react.licence ?? 'MIT', version: react.version ?? '19.0.0' }),
  );
  write(folder, 'node_modules/react/LICENSE', 'React MIT text\n');
  mkdirSync(path.dirname(path.join(folder, NOTICES_PATH)), { recursive: true });

  return folder;
};

const report = (): string => REPORT;

const kinds = (problems: readonly { readonly kind: string }[]): readonly string[] => problems.map(({ kind }) => kind);

describe('run', () => {
  describe('negative cases', () => {
    it('fails the check when no notices were generated', async () => {
      assert.deepEqual(kinds(await run('check', repo(), report)), ['stale-notices']);
    });

    it('fails the check when the committed notices differ', async () => {
      const folder = repo();
      await run('generate', folder, report);
      write(folder, NOTICES_PATH, readFileSync(path.join(folder, NOTICES_PATH), 'utf8').replace('React', 'Other'));

      assert.deepEqual(kinds(await run('check', folder, report)), ['stale-notices']);
    });

    it('refuses an npm production package outside the allowlist and writes nothing', async () => {
      const folder = repo({ licence: 'GPL-3.0-only' });

      assert.deepEqual(kinds(await run('generate', folder, report)), ['licence-not-allowed']);
      assert.throws(() => readFileSync(path.join(folder, NOTICES_PATH)));
    });

    it('refuses a node_modules that does not hold the locked version', async () => {
      assert.deepEqual(kinds(await run('check', repo({ version: '18.0.0' }), report)), ['not-installed']);
    });

    it("refuses an about.toml that differs from deny.toml's allowlist", async () => {
      const folder = repo();
      write(folder, 'about.toml', ABOUT.replace('"ISC", ', ''));

      assert.deepEqual(kinds(await run('check', folder, report)), ['allowlist-differs']);
    });
  });

  describe('positive cases', () => {
    it('writes the Rust and npm production notices, with the MPL-2.0 source, then passes the check', async () => {
      const folder = repo();

      assert.deepEqual(await run('generate', folder, report), []);
      const notices = JSON.parse(readFileSync(path.join(folder, NOTICES_PATH), 'utf8')) as {
        readonly licences: readonly { readonly usedBy: readonly string[] }[];
        readonly sources: readonly { readonly url: string }[];
      };

      assert.deepEqual(
        notices.licences.map(({ usedBy }) => usedBy),
        [['react 19.0.0'], ['serde 1.0.0'], ['selectors 0.1.0']],
      );
      assert.deepEqual(
        notices.sources.map(({ url }) => url),
        ['https://crates.io/crates/selectors/0.1.0'],
      );
      assert.deepEqual(await run('check', folder, report), []);
    });
  });
});
