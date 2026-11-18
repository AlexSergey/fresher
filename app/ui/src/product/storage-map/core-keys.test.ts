import { beforeEach, describe, expect, it } from 'vitest';

import type { MountKind, NodeInfo, PartKind, RemainderReason } from '../../ipc/bindings.generated';

import core from '../../../../../crates/core/locales/en.json';
import { setCoreDictionary, tText } from '../../i18n';
import { notScannedText } from '../place-list/folder-row';
import { partDescription, partName, reasonText } from './parts';

// Every kind of part, the remainder both inside and outside the scan; an other volume without a name of its own.
const PARTS: readonly PartKind[] = [
  { kind: 'explained' },
  { kind: 'your-folders' },
  { kind: 'not-scanned-yet' },
  { kind: 'remainder', outsideScan: true, reasons: [] },
  { kind: 'remainder', outsideScan: false, reasons: [] },
  { kind: 'mac-os' },
  { kind: 'swap-and-sleep' },
  { kind: 'other-volume', name: '' },
  { kind: 'disk-own' },
  { kind: 'free' },
];

const REASONS: readonly RemainderReason[] = [
  { folders: 1, kind: 'unreadable', needs: null },
  { kind: 'snapshots-and-bookkeeping' },
  { kind: 'changed-during-scan' },
];

const NOT_SCANNED: readonly NonNullable<NodeInfo['notScanned']>[] = [
  { kind: 'unreadable', value: 'no-permission' },
  { kind: 'unreadable', value: 'guarded' },
  { kind: 'unreadable', value: 'failed' },
  { kind: 'avoided' },
  { kind: 'other-volume' },
  { kind: 'others-folder' },
  { kind: 'system-items' },
];

// Every kind the bindings know, so a new one fails the lists above until it is added.
const PART_KINDS: Readonly<Record<PartKind['kind'], true>> = {
  'disk-own': true,
  explained: true,
  free: true,
  'mac-os': true,
  'not-scanned-yet': true,
  'other-volume': true,
  remainder: true,
  'swap-and-sleep': true,
  'your-folders': true,
};

const REASON_KINDS: Readonly<Record<RemainderReason['kind'], true>> = {
  'changed-during-scan': true,
  'snapshots-and-bookkeeping': true,
  unreadable: true,
};

const MOUNTS: readonly MountKind[] = ['internal', 'external', 'network'];

// The keys the storage map names as written.
const NAMED = [
  'core.storage.changed',
  'core.storage.place.trash.name',
  'core.storage.place.trash.description',
  'core.place.user-temp.name',
  'core.place.shared-temp.name',
  ...MOUNTS.map((mount) => `core.storage.kind.${mount}`),
];

// A missing key reads as the key itself.
const isKey = (text: null | string): boolean => text === null || text.startsWith('core.');

const folderOf = (notScanned: NodeInfo['notScanned']): NodeInfo => ({
  files: 0,
  folders: 0,
  id: { index: 0, state: 1 },
  kind: 'folder',
  modified: null,
  name: 'folder',
  notScanned,
  owner: null,
  sizes: { allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 },
});

beforeEach(() => {
  setCoreDictionary(core);
});

describe("the storage map's keys in the core's dictionary", () => {
  describe('negative cases', () => {
    it('a key the dictionary does not have reads as the key', () => {
      expect(isKey(tText({ args: {}, key: 'core.storage.part.unknown.name' }))).toBe(true);
    });
  });

  describe('positive cases', () => {
    it('the lists hold every kind of part and of reason', () => {
      expect(new Set(PARTS.map(({ kind }) => kind))).toStrictEqual(new Set(Object.keys(PART_KINDS)));
      expect(new Set(REASONS.map(({ kind }) => kind))).toStrictEqual(new Set(Object.keys(REASON_KINDS)));
    });

    it('every part has a name and a description', () => {
      const missing = PARTS.flatMap((kind) => [partName(kind), partDescription(kind)]).filter(isKey);

      expect(missing).toStrictEqual([]);
    });

    it('every reason of the remainder has a text', () => {
      expect(REASONS.map(reasonText).filter(isKey)).toStrictEqual([]);
    });

    it('every reason a folder was not scanned has a text', () => {
      expect(NOT_SCANNED.map((reason) => notScannedText(folderOf(reason))).filter(isKey)).toStrictEqual([]);
    });

    it('every key named as written is there', () => {
      expect(NAMED.filter((key) => !(key in core))).toStrictEqual([]);
    });
  });
});
