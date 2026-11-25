import { describe, expect, it } from 'vitest';

import type { MapPart, Sizes } from '../ipc/bindings.generated';

import core from '../../../../crates/core/locales/en.json';
import { SAMPLE_STORAGE_MAP, STORAGE_DICTIONARY, SUMMARY_KINDS, withSummary } from './storage-map';

// The ten texts of the disk's five parts, as the core names them (`SummaryKind::name_key`, `description_key`).
const SUMMARY_KEYS = SUMMARY_KINDS.flatMap((kind) => [
  `core.storage.summary.${kind}.name`,
  `core.storage.summary.${kind}.description`,
]);

const sizes = (bytes: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated: bytes,
  completeness,
  logical: bytes,
  reclaimable: bytes,
});

const volumeOf = (parts: readonly MapPart[]): Parameters<typeof withSummary>[0] => ({
  capacity: 0,
  diskChanged: false,
  kind: 'internal',
  name: 'Macintosh HD',
  parts: [...parts],
  places: [],
  purgeable: null,
  roots: [],
});

describe("the disk's five parts", () => {
  describe('negative cases', () => {
    it('a volume without parts has five parts at zero', () => {
      const { summary } = withSummary(volumeOf([]));

      expect(summary).toStrictEqual(SUMMARY_KINDS.map((kind) => ({ kind, sizes: sizes(0) })));
    });

    it('the weakest completeness wins', () => {
      const { summary } = withSummary(
        volumeOf([
          { kind: { kind: 'swap-and-sleep' }, sizes: sizes(3, 'partial') },
          { kind: { kind: 'disk-own' }, sizes: sizes(4, 'unknown') },
        ]),
      );

      expect(summary[3]).toStrictEqual({ kind: 'system-space', sizes: sizes(7, 'unknown') });
    });
  });

  describe('positive cases', () => {
    it("every name and description is in the core's dictionary", () => {
      const texts: Readonly<Record<string, string>> = core;

      expect(SUMMARY_KEYS.filter((key) => !texts[key])).toStrictEqual([]);
      expect(SUMMARY_KEYS.map((key) => STORAGE_DICTIONARY[key])).toStrictEqual(SUMMARY_KEYS.map((key) => texts[key]));
    });

    it("every text the tests and the gallery use is the core's", () => {
      const texts: Readonly<Record<string, string>> = core;
      const different = Object.entries(STORAGE_DICTIONARY).filter(([key, text]) => key in texts && texts[key] !== text);

      expect(different.map(([key]) => key)).toStrictEqual([]);
    });

    it('every part goes to its summary part, in order', () => {
      const { summary } = withSummary(
        volumeOf([
          { kind: { kind: 'explained' }, sizes: sizes(1) },
          { kind: { kind: 'your-folders' }, sizes: sizes(2) },
          { kind: { kind: 'not-scanned-yet' }, sizes: sizes(4) },
          { kind: { kind: 'remainder', outsideScan: true, reasons: [] }, sizes: sizes(8) },
          { kind: { kind: 'mac-os' }, sizes: sizes(16) },
          { kind: { kind: 'swap-and-sleep' }, sizes: sizes(32) },
          { kind: { kind: 'other-volume', name: 'Backup' }, sizes: sizes(64) },
          { kind: { kind: 'disk-own' }, sizes: sizes(128) },
          { kind: { kind: 'free' }, sizes: sizes(256) },
        ]),
      );

      expect(summary.map(({ kind, sizes: { allocated } }) => [kind, allocated])).toStrictEqual([
        ['can-be-freed', 1],
        ['files-and-apps', 2 + 4 + 8 + 64],
        ['mac-os', 16],
        ['system-space', 32 + 128],
        ['free', 256],
      ]);
    });

    it("the sample volume's summary adds up to its capacity", () => {
      const [volume] = SAMPLE_STORAGE_MAP.volumes;

      expect(volume?.summary.reduce((sum, { sizes: { allocated } }) => sum + allocated, 0)).toBe(volume?.capacity);
    });
  });
});
