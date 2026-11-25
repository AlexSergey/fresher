import { beforeEach, describe, expect, it } from 'vitest';

import type { SettingDescriptor } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { hitsOn, type SearchEntry, searchSettings, settingsEntries } from './settings-search';

const entry = (label: string, description: string, module = 'demo', key: null | string = label): SearchEntry => ({
  description,
  key,
  label,
  page: { kind: 'module', module },
  place: 'Debug › Demo module',
});

const ENTRIES: readonly SearchEntry[] = [
  entry('Old after', '90 days. A project untouched this long counts as old.'),
  entry('Example findings', 'How many made-up findings the demo module shows.'),
  { description: '', key: null, label: 'Temporary files', page: { kind: 'module', module: 'temp' }, place: 'System' },
];

const descriptor = (
  key: string,
  kind: SettingDescriptor['kind'],
  value: SettingDescriptor['default'],
): SettingDescriptor => ({ default: value, effect: 'next-scan', group: 'scan', key, kind });

describe('searchSettings', () => {
  describe('negative cases', () => {
    it('an empty or blank query matches nothing', () => {
      expect(searchSettings('', ENTRIES)).toEqual([]);
      expect(searchSettings('   ', ENTRIES)).toEqual([]);
    });

    it('finds nothing in a text no entry has', () => {
      expect(searchSettings('zebra', ENTRIES)).toEqual([]);
    });

    it('a section matches by its name only', () => {
      const hits = searchSettings('files', ENTRIES).map(({ entry: { label } }) => label);

      expect(hits).toEqual(['Temporary files']);
    });
  });

  describe('positive cases', () => {
    it('matches a label, a description and a section name, any case', () => {
      expect(searchSettings('OLD AFTER', ENTRIES).map(({ entry: { label } }) => label)).toEqual(['Old after']);
      expect(searchSettings('MADE-UP', ENTRIES).map(({ entry: { label } }) => label)).toEqual(['Example findings']);
      expect(searchSettings('temporary', ENTRIES).map(({ entry: { label } }) => label)).toEqual(['Temporary files']);
    });

    it('trims the query', () => {
      expect(searchSettings('  days  ', ENTRIES)).toHaveLength(1);
    });

    it('hits come in the tree order', () => {
      const labels = searchSettings('e', ENTRIES).map(({ entry: { label } }) => label);

      expect(labels).toEqual(['Old after', 'Example findings', 'Temporary files']);
    });

    it('ranges mark every match', () => {
      const [hit] = searchSettings('old', ENTRIES);

      expect(hit?.labelRanges).toEqual([[0, 3]]);
      expect(hit?.descriptionRanges).toEqual([[49, 52]]);
    });

    it('counts the hits of a page', () => {
      const hits = searchSettings('e', ENTRIES);

      expect(hitsOn(hits, { kind: 'module', module: 'demo' })).toBe(2);
      expect(hitsOn(hits, { kind: 'module', module: 'temp' })).toBe(1);
      expect(hitsOn(hits, { kind: 'module', module: 'logs' })).toBe(0);
      expect(hitsOn(hits, { kind: 'general' })).toBe(0);
    });
  });
});

describe('settingsEntries', () => {
  beforeEach(() => {
    setCoreDictionary({
      'core.group.debug.name': 'Debug',
      'core.scan.stale-after.description': 'Untouched this long counts as stale.',
      'core.scan.stale-after.label': 'Stale after',
      'demo.count.description': 'How many findings.',
      'demo.count.label': 'Example findings',
      'demo.enabled.label': 'Demo module',
    });
  });

  const stale = descriptor(
    'core.scan.stale-after',
    { kind: 'number', max: 365, min: 1, unit: 'days' },
    { kind: 'number', value: 90 },
  );
  const count = descriptor('demo.count', { kind: 'number', max: 50, min: 1, unit: null }, { kind: 'number', value: 8 });
  const enabled = descriptor('demo.enabled', { kind: 'switch' }, { kind: 'switch', value: true });
  const groups = {
    general: [],
    modules: [{ group: 'debug', items: [{ module: 'demo', rows: [count], switch: enabled }] }],
    scan: [stale],
  };

  describe('negative cases', () => {
    it('a setting no page shows is not searched', () => {
      const hidden = descriptor('core.hidden', { kind: 'switch' }, { kind: 'switch', value: false });
      const keys = settingsEntries({ ...groups, general: [] }, {}).map(({ key }) => key);

      expect(keys).not.toContain(hidden.key);
    });

    it('a setting that is not a number has no value and no space in front of its description', () => {
      const plain = descriptor('demo.count', { kind: 'switch' }, { kind: 'switch', value: true });
      const found = settingsEntries({ ...groups, general: [plain] }, {}).find(({ key }) => key === 'demo.count');

      expect(found?.description).toBe('How many findings.');
    });
  });

  describe('positive cases', () => {
    it('leads a number with its current value and unit', () => {
      const found = settingsEntries(groups, { 'core.scan.stale-after': { kind: 'number', value: 1 } });

      expect(found.find(({ key }) => key === 'core.scan.stale-after')?.description).toBe(
        '1 day. Untouched this long counts as stale.',
      );
      expect(found.find(({ key }) => key === 'demo.count')?.description).toBe('8. How many findings.');
    });

    it('lists the sections, then the core settings, then the modules', () => {
      const labels = settingsEntries(groups, {}).map(({ label }) => label);

      expect(labels.slice(0, 3)).toEqual(['General', 'Scan', 'Stale after']);
      expect(labels.slice(-3)).toEqual(['Demo module', 'Example findings', 'Look for Demo module']);
    });

    it('puts a module setting in "Group › Module"', () => {
      const found = settingsEntries(groups, {}).find(({ key }) => key === 'demo.count');

      expect(found?.place).toBe('Debug › Demo module');
    });
  });
});
