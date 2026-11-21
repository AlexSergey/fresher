import { beforeEach, describe, expect, it } from 'vitest';

import type {
  CategoryTotal,
  DangerLevel,
  MapPart,
  MeaningGroup,
  PartKind,
  Sizes,
  VolumeMap,
} from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { STORAGE_DICTIONARY } from '../../test/storage-map';
import { levelShares, ringParts, ringSegments } from './ring-parts';

const sizes = (bytes: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated: bytes,
  completeness,
  logical: bytes,
  reclaimable: bytes,
});

const total = (category: string, highestDanger: CategoryTotal['highestDanger'], bytes: number): CategoryTotal => ({
  category,
  count: 1,
  highestDanger,
  sizes: sizes(bytes),
});

const part = (kind: PartKind, bytes: number): MapPart => ({ kind, sizes: sizes(bytes) });

const volumeOf = (parts: readonly MapPart[]): VolumeMap => ({
  capacity: 0,
  diskChanged: false,
  kind: 'internal',
  name: 'Macintosh HD',
  parts: [...parts],
  places: [],
  purgeable: null,
  roots: [],
});

const TOTALS = [total('a.caches', 'safe', 100), total('a.logs', 'safe', 50), total('a.builds', 'high', 30)];

const category = (id: string, bytes: number, completeness: Sizes['completeness'] = 'complete'): MeaningGroup => ({
  kind: 'category',
  value: { category: id, sizes: sizes(bytes, completeness) },
});

const bytesOf = (levels: ReadonlyMap<DangerLevel, Sizes>, level: DangerLevel): number | undefined =>
  levels.get(level)?.allocated;

beforeEach(() => {
  setCoreDictionary(STORAGE_DICTIONARY);
});

describe('levelShares', () => {
  describe('negative cases', () => {
    it('leaves out a share whose category the totals do not know', () => {
      const levels = levelShares([category('a.caches', 70), category('a.ignored', 999)], TOTALS);

      expect(levels.get('safe')?.allocated).toBe(70);
      expect(levels.size).toBe(1);
    });

    it('draws no level with nothing in it', () => {
      const levels = levelShares([category('a.caches', 70)], TOTALS);

      expect(levels.has('high')).toBe(false);
      expect(levels.has('low')).toBe(false);
    });

    it('falls back to the totals while the groups hold only parts', () => {
      const levels = levelShares([{ kind: 'part', value: part({ kind: 'free' }, 9) }], TOTALS);

      expect(bytesOf(levels, 'safe')).toBe(150);
      expect(bytesOf(levels, 'high')).toBe(30);
      expect(bytesOf(levelShares(null, TOTALS), 'safe')).toBe(150);
    });
  });

  describe('positive cases', () => {
    it("sums the category shares and the apps' category shares by the totals' level", () => {
      const app: MeaningGroup = {
        kind: 'app',
        value: {
          app: { id: 'x', installed: true, name: 'X' },
          categories: [
            { category: 'a.logs', sizes: sizes(11) },
            { category: 'a.builds', sizes: sizes(5) },
          ],
          data: [],
          sizes: sizes(16),
        },
      };
      const levels = levelShares([category('a.caches', 70), app], TOTALS);

      expect(bytesOf(levels, 'safe')).toBe(81);
      expect(bytesOf(levels, 'high')).toBe(5);
    });

    it('keeps the weakest completeness', () => {
      const levels = levelShares([category('a.caches', 70), category('a.logs', 3, 'partial')], TOTALS);

      expect(levels.get('safe')?.completeness).toBe('partial');
    });
  });
});

describe('ringParts', () => {
  const volume = volumeOf([
    part({ kind: 'explained' }, 999),
    part({ kind: 'your-folders' }, 200),
    part({ kind: 'not-scanned-yet' }, 0),
    part({ kind: 'mac-os' }, 23),
    part({ kind: 'free' }, 109),
  ]);
  const levels = levelShares(null, TOTALS);

  describe('negative cases', () => {
    it('drops a part of zero size', () => {
      expect(ringParts(volume, levels).map(({ key }) => key)).not.toContain('not-scanned-yet-2');
    });

    it('does not draw Explained itself: its levels stand for it', () => {
      expect(ringParts(volume, levels).map(({ key }) => key)).not.toContain('explained-0');
    });
  });

  describe('positive cases', () => {
    it("puts the levels first, Safe to High, then the parts in the core's order, Free last", () => {
      expect(ringParts(volume, levels).map(({ key }) => key)).toStrictEqual([
        'level-safe',
        'level-high',
        'your-folders-1',
        'mac-os-3',
        'free-4',
      ]);
    });

    it('names a level by its zone and a part by the core', () => {
      expect(ringParts(volume, levels).map(({ label }) => label)).toStrictEqual([
        'Safe to remove',
        'High risk',
        'Your folders',
        'macOS',
        'Free',
      ]);
    });

    it('gives each its own tone and its explanation', () => {
      const parts = ringParts(volume, levels);

      expect(parts.map(({ tone }) => tone)).toStrictEqual(['safe', 'high', 'your-folders', 'mac-os', 'free']);
      expect(parts[0]?.about).toStrictEqual({ kind: 'level', level: 'safe' });
      expect(parts[2]?.about.kind).toBe('part');
    });

    it('turns the parts into segments sized by their allocated bytes', () => {
      expect(ringSegments(ringParts(volume, levels)).map(({ key, value }) => [key, value])).toStrictEqual([
        ['level-safe', 150],
        ['level-high', 30],
        ['your-folders-1', 200],
        ['mac-os-3', 23],
        ['free-4', 109],
      ]);
    });
  });
});
