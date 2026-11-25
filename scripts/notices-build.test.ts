import assert from 'node:assert/strict';
import { describe, it } from 'node:test';

import {
  buildNotices,
  type CargoAboutLicence,
  checkAboutConfig,
  chooseLicences,
  type NoticeSource,
  npmEntries,
  type NpmPackage,
  parsePolicy,
  type Policy,
  rustEntries,
} from './notices-build.ts';

const DENY = `[advisories]
ignore = []

[licenses]
allow = ["Apache-2.0", "ISC", "MIT"]
confidence-threshold = 0.93
exceptions = [
  { crate = "selectors", allow = [
    "MPL-2.0",
  ] },
]

[licenses.private]
ignore = true
`;

const ABOUT = `ignore-dev-dependencies = true
accepted = ["MIT", "ISC", "Apache-2.0"]

[selectors]
accepted = ["MPL-2.0"]
`;

const POLICY: Policy = { allow: ['Apache-2.0', 'ISC', 'MIT'], exceptions: new Map([['selectors', ['MPL-2.0']]]) };

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

const npm = (name: string, declared: unknown, files: Record<string, string>): NpmPackage => ({
  files: new Map(Object.entries(files)),
  licence: declared,
  name,
  version: '1.0.0',
});

describe('parsePolicy', () => {
  describe('positive cases', () => {
    it("reads deny.toml's allowlist and its exceptions by crate", () => {
      assert.deepEqual(parsePolicy(DENY), POLICY);
    });
  });
});

describe('checkAboutConfig', () => {
  describe('negative cases', () => {
    it('refuses an accepted list that differs from the allowlist', () => {
      const problems = checkAboutConfig(ABOUT.replace('"ISC", ', ''), POLICY);

      assert.deepEqual(
        problems.map(({ kind }) => kind),
        ['allowlist-differs'],
      );
    });

    it('refuses an exception missing from about.toml', () => {
      assert.equal(checkAboutConfig(ABOUT.replace(/\[selectors\][\s\S]*/u, ''), POLICY).length, 1);
    });

    it('refuses an exception under another licence', () => {
      assert.equal(checkAboutConfig(ABOUT.replace('"MPL-2.0"', '"LGPL-2.1"'), POLICY).length, 1);
    });
  });

  describe('positive cases', () => {
    it('takes the same licences in any order', () => {
      assert.deepEqual(checkAboutConfig(ABOUT, POLICY), []);
    });
  });
});

describe('chooseLicences', () => {
  const allow = ['Apache-2.0', 'MIT'];

  describe('negative cases', () => {
    it('refuses a licence outside the allowlist', () => {
      assert.equal(chooseLicences('GPL-3.0-only', allow), null);
    });

    it('refuses an AND with one licence outside it', () => {
      assert.equal(chooseLicences('MIT AND GPL-3.0-only', allow), null);
    });

    it('refuses a licence WITH an exception', () => {
      assert.equal(chooseLicences('Apache-2.0 WITH LLVM-exception', allow), null);
    });

    it('refuses an expression that is not well formed', () => {
      assert.equal(chooseLicences('(MIT OR Apache-2.0', allow), null);
      assert.equal(chooseLicences('MIT OR', allow), null);
      assert.equal(chooseLicences('MIT Apache-2.0', allow), null);
    });
  });

  describe('positive cases', () => {
    it('takes a single licence', () => {
      assert.deepEqual(chooseLicences('MIT', allow), ['MIT']);
    });

    it('takes the first allowed alternative of an OR', () => {
      assert.deepEqual(chooseLicences('GPL-2.0-only OR Apache-2.0 OR MIT', allow), ['Apache-2.0']);
    });

    it('takes every licence of an AND, in parentheses too', () => {
      assert.deepEqual(chooseLicences('(GPL-2.0-only OR MIT) AND Apache-2.0', allow), ['MIT', 'Apache-2.0']);
    });
  });
});

describe('rustEntries', () => {
  describe('negative cases', () => {
    it('refuses a licence outside the allowlist for a crate that is no exception', () => {
      const { entries, problems } = rustEntries(
        { licenses: [licence('MPL-2.0', 'MPL', 'MPL', [['other', '1.0.0']])] },
        POLICY,
      );

      assert.deepEqual(entries, []);
      assert.deepEqual(
        problems.map(({ kind, where }) => [kind, where]),
        [['licence-not-allowed', 'crate other 1.0.0']],
      );
    });
  });

  describe('positive cases', () => {
    it('names an exception crate with the address of its source', () => {
      const { problems, sources } = rustEntries(
        { licenses: [licence('MPL-2.0', 'MPL', 'MPL', [['selectors', '0.2.0']])] },
        POLICY,
      );

      assert.deepEqual(problems, []);
      assert.deepEqual(sources, [
        { licence: 'MPL-2.0', name: 'selectors', url: 'https://crates.io/crates/selectors/0.2.0', version: '0.2.0' },
      ]);
    });

    it('gives an allowed licence its full name', () => {
      const { entries } = rustEntries(
        { licenses: [licence('MIT', 'whatever', 'MIT text', [['serde', '1.0.0']])] },
        POLICY,
      );

      assert.deepEqual(entries, [{ id: 'MIT', name: 'MIT License', part: 'serde 1.0.0', text: 'MIT text' }]);
    });
  });
});

describe('npmEntries', () => {
  describe('negative cases', () => {
    it('refuses a package with no licence', () => {
      const { problems } = npmEntries([npm('a', undefined, {}), npm('b', { type: 'MIT' }, {})], POLICY);

      assert.deepEqual(
        problems.map(({ kind }) => kind),
        ['no-licence', 'no-licence'],
      );
    });

    it('refuses a licence outside the allowlist, an exception of a crate included', () => {
      const { problems } = npmEntries(
        [npm('a', 'GPL-3.0-only', { LICENSE: 'x' }), npm('selectors', 'MPL-2.0', { LICENSE: 'x' })],
        POLICY,
      );

      assert.deepEqual(
        problems.map(({ kind, where }) => [kind, where]),
        [
          ['licence-not-allowed', 'npm a 1.0.0'],
          ['licence-not-allowed', 'npm selectors 1.0.0'],
        ],
      );
    });

    it('refuses a package without the licence file it needs', () => {
      const none = npmEntries([npm('a', 'MIT', {})], POLICY);
      const other = npmEntries(
        [npm('b', 'MIT', { 'LICENSE-APACHE': 'x', 'LICENSE-ISC': 'z', 'LICENSE.spdx': 'y' })],
        POLICY,
      );

      assert.deepEqual(
        [...none.problems, ...other.problems].map(({ kind }) => kind),
        ['no-licence-file', 'no-licence-file'],
      );
    });
  });

  describe('positive cases', () => {
    it("takes a package's only licence file", () => {
      const { entries, problems } = npmEntries([npm('react', 'MIT', { LICENSE: 'MIT text' })], POLICY);

      assert.deepEqual(problems, []);
      assert.deepEqual(entries, [{ id: 'MIT', name: 'MIT License', part: 'react 1.0.0', text: 'MIT text' }]);
    });

    it('takes the file named for the chosen licence, past an SPDX file', () => {
      const { entries } = npmEntries(
        [
          npm('@tauri-apps/api', 'Apache-2.0 OR MIT', {
            'LICENSE-APACHE-2.0': 'Apache text',
            'LICENSE-MIT': 'MIT text',
            'LICENSE.spdx': 'SPDX',
          }),
        ],
        POLICY,
      );

      assert.deepEqual(
        entries.map(({ id, text }) => [id, text]),
        [['Apache-2.0', 'Apache text']],
      );
    });
  });
});

describe('buildNotices', () => {
  describe('positive cases', () => {
    it('groups the parts of the same text, sorted, whatever the line ends and trailing spaces', () => {
      const notices = buildNotices(
        [
          { id: 'MIT', name: 'MIT License', part: 'zeta 1.0.0', text: '\nMIT  \r\ntext\n\n' },
          { id: 'MIT', name: 'MIT License', part: 'alpha 1.0.0', text: 'MIT\ntext' },
          { id: 'MIT', name: 'MIT License', part: 'alpha 1.0.0', text: 'MIT\ntext' },
          { id: 'MIT', name: 'MIT License', part: 'beta 1.0.0', text: 'another MIT text' },
          { id: 'Apache-2.0', name: 'Apache License 2.0', part: 'gamma 1.0.0', text: 'Apache' },
        ],
        [],
      );

      assert.deepEqual(notices.licences, [
        { id: 'Apache-2.0', name: 'Apache License 2.0', text: 'Apache', usedBy: ['gamma 1.0.0'] },
        { id: 'MIT', name: 'MIT License', text: 'MIT\ntext', usedBy: ['alpha 1.0.0', 'zeta 1.0.0'] },
        { id: 'MIT', name: 'MIT License', text: 'another MIT text', usedBy: ['beta 1.0.0'] },
      ]);
    });

    it('lists each source once, by name and version', () => {
      const source = (name: string, version: string): NoticeSource => ({
        licence: 'MPL-2.0',
        name,
        url: `u/${name}`,
        version,
      });
      const notices = buildNotices([], [source('b', '1'), source('a', '2'), source('a', '1'), source('b', '1')]);

      assert.deepEqual(
        notices.sources.map(({ name, version }) => `${name} ${version}`),
        ['a 1', 'a 2', 'b 1'],
      );
    });
  });
});
