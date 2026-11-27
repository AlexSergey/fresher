import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { describe, it } from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  buildSection,
  checkFlags,
  type Flag,
  isReadingSource,
  notesSection,
  readFlags,
  resolveRelease,
  type Source,
} from './flags.ts';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

const flag = (name: string, fields: Flag['fields'] = {}): Flag => ({
  fields: {
    about: 'Whether it is shown.',
    development: 'shown',
    release: 'hidden',
    type: 'Shown',
    values: ['shown', 'hidden'],
    ...fields,
  },
  name,
});

const reading = (name: string): Source[] => [{ path: 'app/ui/src/app.tsx', text: `if (flags.${name}) {}` }];

const messages = (flags: Flag[], sources: Source[]): string[] => checkFlags(flags, sources).map((v) => v.message);

const FILE = [
  '# a comment',
  '',
  '[high]',
  'about = "Whether High is offered."',
  'type = "HighLevel"',
  'values = ["offered", "withheld"]',
  'release = "withheld"',
  'development = "offered"',
  '',
  '[largest]',
  'about = "Whether Largest is shown."',
  'type = "LargestScreen"',
  'values = ["shown", "hidden"]',
  'release = "hidden"',
  'development = "shown"',
  '',
].join('\n');

describe('flags', () => {
  describe('negative cases', () => {
    it('refuses a line the reader does not know', () => {
      const inline = readFlags('[a]\nabout = { x = 1 }\n');
      assert.match(inline.violations[0]?.message ?? '', /flags\.toml:2: cannot read/u);
      const number = readFlags('[a]\nabout = 1\n');
      assert.match(number.violations[0]?.message ?? '', /flags\.toml:2: cannot read/u);
    });

    it('refuses a table missing a key or holding another', () => {
      const rest = Object.fromEntries(Object.entries(flag('a').fields).filter(([key]) => key !== 'about'));
      assert.deepEqual(messages([flag('a', { extra: 'x' })], reading('a')), ['[a] has an unknown key "extra"']);
      assert.deepEqual(messages([{ fields: rest, name: 'a' }], reading('a')), ['[a] has no "about"']);
    });

    it('refuses an about that is empty, has no full stop or is too long', () => {
      for (const about of ['', 'No full stop', `${'x'.repeat(120)}.`, 'One. Two.']) {
        assert.equal(messages([flag('a', { about })], reading('a')).length, 1, about);
      }
    });

    it('refuses fewer than two values, a repeated one or one not in kebab-case', () => {
      assert.match(
        messages([flag('a', { development: 'x', release: 'x', values: ['x'] })], reading('a'))[0] ?? '',
        /at least two/u,
      );
      assert.match(
        messages([flag('a', { values: ['shown', 'hidden', 'hidden'] })], reading('a')).join(),
        /"hidden" is repeated/u,
      );
      assert.match(messages([flag('a', { values: ['shown', 'Hidden'] })], reading('a')).join(), /not kebab-case/u);
    });

    it('refuses a release or development value outside its values', () => {
      assert.deepEqual(messages([flag('a', { release: 'other' })], reading('a')), [
        '[a] release "other" is not among its values',
      ]);
      assert.deepEqual(messages([flag('a', { development: 'other' })], reading('a')), [
        '[a] development "other" is not among its values',
      ]);
    });

    it('refuses a flag name or a type that is not an identifier of its kind', () => {
      assert.match(messages([flag('Bad-name')], reading('Bad-name'))[0] ?? '', /name must match/u);
      assert.match(messages([flag('a', { type: 'bad_type' })], reading('a'))[0] ?? '', /PascalCase/u);
    });

    it('refuses a flag no source reads', () => {
      const unread: Source[] = [
        { path: 'app/ui/src/a.test.tsx', text: 'flags.a' },
        { path: 'app/ui/src/a.stories.tsx', text: 'flags.a' },
        { path: 'app/ui/src/ipc/bindings.generated.ts', text: 'flags.a' },
        { path: 'app/tauri/src/sandbox/tests.rs', text: 'FLAGS.a' },
      ];
      assert.equal(messages([flag('a')], unread).length, 1);
      assert.equal(messages([flag('a')], []).length, 1);
    });

    it('refuses an override naming an unknown flag, a value outside its values, a flag twice or no equals sign', () => {
      const flags = [flag('a')];
      assert.match(resolveRelease(flags, 'b=shown').errors[0] ?? '', /there is no flag `b`/u);
      assert.match(resolveRelease(flags, 'a=other').errors[0] ?? '', /`other` is not a value of `a`/u);
      assert.match(resolveRelease(flags, 'a=shown, a=hidden').errors[0] ?? '', /`a` is named twice/u);
      assert.match(resolveRelease(flags, 'a').errors[0] ?? '', /`a` has no `=`/u);
    });

    it('gives no section for a file the reader refuses or an override it refuses', () => {
      const refused = buildSection(FILE.replace('release = "hidden"', 'release = "hidden" # note'), undefined);
      assert.equal(refused.section, undefined);
      assert.match(refused.problems[0] ?? '', /cannot read/u);
      const override = buildSection(FILE, 'nope=1');
      assert.equal(override.section, undefined);
      assert.match(override.problems[0] ?? '', /there is no flag/u);
    });
  });

  describe('positive cases', () => {
    it("accepts the repository's flags.toml against the repository's sources", () => {
      const { flags, violations } = readFlags(readFileSync(join(ROOT, 'flags.toml'), 'utf8'));
      const sources = ['app/tauri/src', 'app/ui/src'].flatMap((dir) =>
        readdirSync(join(ROOT, dir), { recursive: true, withFileTypes: true })
          .filter((entry) => entry.isFile())
          .map((entry) => join(entry.parentPath, entry.name))
          .filter((path) => isReadingSource(relative(ROOT, path)))
          .map((path) => ({ path: relative(ROOT, path), text: readFileSync(path, 'utf8') })),
      );
      assert.deepEqual(violations, []);
      assert.ok(flags.length > 0);
      assert.deepEqual(checkFlags(flags, sources), []);
    });

    it('takes the release column when no override is set', () => {
      const { flags } = readFlags(FILE);
      for (const overrides of [undefined, '', ' , ']) {
        const { errors, resolved } = resolveRelease(flags, overrides);
        assert.deepEqual(errors, []);
        assert.deepEqual(
          resolved.map((r) => r.value),
          ['withheld', 'hidden'],
        );
      }
    });

    it('an override replaces only the flags it names', () => {
      const { flags } = readFlags(FILE);
      const { errors, resolved } = resolveRelease(flags, ' largest = shown ,');
      assert.deepEqual(errors, []);
      assert.deepEqual(
        resolved.map((r) => r.value),
        ['withheld', 'shown'],
      );
    });

    it("prints one line per flag in the file's order", () => {
      const { flags } = readFlags(FILE);
      assert.equal(
        notesSection(resolveRelease(flags, undefined).resolved),
        '## This build\n\n- `high`: withheld — Whether High is offered.\n- `largest`: hidden — Whether Largest is shown.\n',
      );
    });
  });
});
