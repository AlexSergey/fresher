import assert from 'node:assert/strict';
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, describe, it } from 'node:test';

import { checkDictionary, type Dictionary, hasDictionary, type RuleId } from './check-texts.ts';

const rulesOf = (dictionary: Dictionary): RuleId[] => checkDictionary('en.json', dictionary).map((v) => v.rule);

const CATEGORY: Dictionary = {
  'm.c.comes-back': 'Only if an app makes them again.',
  'm.c.if-removed': 'They go to the Trash.',
  'm.c.line': 'Files nothing has used',
  'm.c.made-by': 'Many apps.',
  'm.c.name': 'Old files',
  'm.c.what': 'Files nothing has used.',
};

const folder = mkdtempSync(join(tmpdir(), 'check-texts-'));

after(() => {
  rmSync(folder, { force: true, recursive: true });
});

describe('check-texts', () => {
  describe('negative cases', () => {
    it('skips a module without a dictionary', () => {
      assert.equal(hasDictionary(join(folder, 'missing', 'en.json')), false);
    });

    it('throws when a dictionary cannot be read for another reason', () => {
      const unreadable = join(folder, 'a-folder.json');
      mkdirSync(unreadable);

      assert.throws(() => hasDictionary(unreadable), { code: 'EISDIR' });
    });

    it('gives a key whose segment is a property of every object the other limit', () => {
      assert.deepEqual(rulesOf({ 'x.constructor': `${'a'.repeat(241)}.` }), ['length']);
      assert.deepEqual(rulesOf({ 'x.constructor': `${'a'.repeat(239)}.` }), []);
    });

    it('reports an exclamation mark', () => {
      assert.deepEqual(rulesOf({ 'a.b': 'Done!' }), ['exclamation']);
    });

    it('reports a name over 48 characters', () => {
      assert.deepEqual(rulesOf({ 'a.name': 'x'.repeat(49) }), ['length']);
    });

    it('counts a plural by the segment before it', () => {
      assert.deepEqual(rulesOf({ 'a.name.one': 'x'.repeat(49), 'a.name.other': 'x'.repeat(48) }), ['length']);
    });

    it('reports a banned word as a whole word only', () => {
      assert.deepEqual(rulesOf({ 'a.b': 'Please wait, Click Here.' }), ['banned', 'banned']);
      assert.deepEqual(rulesOf({ 'a.b': 'A simplifiedly directorynote.' }), []);
    });

    it('reports delete outside a permanent delete', () => {
      assert.deepEqual(rulesOf({ 'a.b': 'Delete it.' }), ['delete']);
    });

    it('reports a lower-case trash', () => {
      assert.deepEqual(rulesOf({ 'a.b': 'It goes to the trash.' }), ['trash']);
    });

    it('reports a label without a description outside ui', () => {
      assert.deepEqual(rulesOf({ 'a.x.label': 'X' }), ['description']);
    });

    it('reports a category missing a part', () => {
      const partial = Object.fromEntries(Object.entries(CATEGORY).filter(([key]) => key !== 'm.c.comes-back'));

      assert.deepEqual(rulesOf(partial), ['category-parts', 'category-parts', 'category-parts']);
    });

    it('reports a category missing its line', () => {
      const partial = Object.fromEntries(Object.entries(CATEGORY).filter(([key]) => key !== 'm.c.line'));

      assert.deepEqual(rulesOf(partial), ['category-parts', 'category-parts', 'category-parts', 'category-parts']);
    });

    it('reports a line over 50 characters', () => {
      assert.deepEqual(rulesOf({ ...CATEGORY, 'm.c.line': 'x'.repeat(51) }), ['length']);
    });

    it('reports a line ending with a full stop', () => {
      assert.deepEqual(rulesOf({ ...CATEGORY, 'm.c.line': 'Files nothing has used.' }), ['form']);
    });

    it('reports a category name of five words', () => {
      assert.deepEqual(rulesOf({ ...CATEGORY, 'm.c.name': 'Old files of your apps' }), ['name-words']);
    });

    it('reports a double space', () => {
      assert.deepEqual(rulesOf({ 'a.b': 'Two  spaces' }), ['form']);
      assert.deepEqual(rulesOf({ 'a.b': ' padded' }), ['form']);
    });

    it('reports a lower-case trash next to a placeholder', () => {
      assert.deepEqual(rulesOf({ 'ui.a.b': 'Put it in the trash · {trash}' }), ['trash']);
    });

    it('reports an unterminated sentence key outside ui and inside ui when it is not exempt', () => {
      assert.deepEqual(rulesOf({ 'a.b.failed': 'It failed' }), ['sentence']);
      assert.deepEqual(rulesOf({ 'ui.a.failed': 'It failed' }), ['sentence']);
    });

    it('reports a description that does not end as a sentence', () => {
      assert.deepEqual(rulesOf({ 'a.x.description': 'Does a thing', 'a.x.label': 'X' }), ['sentence']);
    });
  });

  describe('positive cases', () => {
    it('passes a line of 50 characters', () => {
      assert.deepEqual(rulesOf({ ...CATEGORY, 'm.c.line': 'x'.repeat(50) }), []);
    });

    it('passes a four-word name', () => {
      assert.deepEqual(rulesOf({ ...CATEGORY, 'm.c.name': 'Old files of apps' }), []);
    });

    it('does not count the words of a ui name', () => {
      assert.deepEqual(rulesOf({ 'ui.a.name': 'One two three four five six' }), []);
    });

    it('does not read a placeholder name as a lower-case trash', () => {
      assert.deepEqual(rulesOf({ 'ui.a.b': 'Sandbox · {trash}' }), []);
    });

    it('does not ask a ui label to be a category part', () => {
      assert.deepEqual(rulesOf({ 'ui.a.what': 'What it is.' }), []);
    });

    it('passes a dictionary that keeps every rule', () => {
      assert.deepEqual(
        rulesOf({ ...CATEGORY, 'a.b': 'The Trash.', 'a.x.description': 'Does a thing.', 'a.x.label': 'X' }),
        [],
      );
    });

    it('lets ui labels go without a description', () => {
      assert.deepEqual(rulesOf({ 'ui.scan.label': 'Scan' }), []);
    });

    it('lets a part label with its effect go without a description', () => {
      assert.deepEqual(rulesOf({ 'd.part.cache.effect': 'Frees space.', 'd.part.cache.label': 'Cache' }), []);
    });

    it('accepts delete in a permanent text', () => {
      assert.deepEqual(rulesOf({ 'a.b': 'Delete permanently.', 'a.delete': 'Delete it.' }), []);
    });

    it('finds a dictionary that is there', () => {
      const dictionary = join(folder, 'en.json');
      writeFileSync(dictionary, '{}');

      assert.equal(hasDictionary(dictionary), true);
    });

    it('skips an exempt key', () => {
      const exempt = new Map([['exempt.key', 'test']]);

      assert.deepEqual(checkDictionary('en.json', { 'exempt.key': 'Bad!  trash' }, exempt), []);
    });
  });
});
