import type { SettingDescriptor, SettingKey, SettingValue } from '../../ipc/bindings.generated';
import type { SettingsPage } from '../../state/navigation';
import type { SettingsGroups } from './settings-groups';

import { t, tText } from '../../i18n';
import { settingText } from '../../product/settings-row/texts';

// A match: where its text starts and ends.
export type Range = readonly [start: number, end: number];

// What the search looks through: a setting (`key`) or a section (no key), the page that holds it, where it sits and
// the texts the row or the section shows.
export type SearchEntry = {
  readonly description: string;
  readonly key: null | SettingKey;
  readonly label: string;
  readonly page: SettingsPage;
  // "Group › Module", a core section's name, or empty for a section's own entry.
  readonly place: string;
};

export type SearchHit = {
  readonly descriptionRanges: readonly Range[];
  readonly entry: SearchEntry;
  readonly labelRanges: readonly Range[];
};

const CORE_SECTIONS = ['general', 'scan', 'ignored', 'permissions', 'about'] as const;

const SECTION_NAMES = {
  about: 'ui.screen.settings.group.about',
  general: 'ui.screen.settings.group.general',
  ignored: 'ui.screen.settings.group.ignored',
  permissions: 'ui.screen.settings.group.permissions',
  scan: 'ui.screen.settings.group.scan',
} as const;

// Every non-overlapping place `needle` (already lower case) occurs in `text`, ignoring case.
const rangesOf = (text: string, needle: string): readonly Range[] => {
  const lower = text.toLowerCase();
  const found: Range[] = [];

  for (let at = lower.indexOf(needle); at !== -1; at = lower.indexOf(needle, at + needle.length)) {
    found.push([at, at + needle.length]);
  }

  return found;
};

// The entries that match `query` (trimmed, any case), in the order `entries` came, with what matched marked. A
// setting matches by its label or its description, a section by its name; a blank query matches none.
export const searchSettings = (query: string, entries: readonly SearchEntry[]): readonly SearchHit[] => {
  const needle = query.trim().toLowerCase();

  if (needle === '') {
    return [];
  }

  return entries.flatMap((entry) => {
    const labelRanges = rangesOf(entry.label, needle);
    const descriptionRanges = entry.key === null ? [] : rangesOf(entry.description, needle);

    return labelRanges.length === 0 && descriptionRanges.length === 0
      ? []
      : [{ descriptionRanges, entry, labelRanges }];
  });
};

const count = (value: number): { readonly kind: 'count'; readonly value: number } => ({ kind: 'count', value });

// A number's current value, first in its description: "90 days.".
const valueLead = ({ kind }: SettingDescriptor, value: SettingValue): string => {
  if (kind.kind !== 'number' || value.kind !== 'number') {
    return '';
  }

  switch (kind.unit) {
    case 'bytes':
      return t('ui.screen.settings.value.bytes', { size: { kind: 'size', value: value.value } });
    case 'days':
      return t('ui.screen.settings.value.days', { count: count(value.value) });
    case null:
      return t('ui.screen.settings.value.plain', { count: count(value.value) });
  }
};

// Every setting shown in Settings and every section, in the order of the tree: the core's sections, then the modules
// by group. A setting that no page shows is not in it.
export const settingsEntries = (
  groups: SettingsGroups,
  values: Readonly<Record<string, SettingValue>>,
): readonly SearchEntry[] => {
  const ofSetting = (descriptor: SettingDescriptor, place: string, page: SettingsPage): SearchEntry => ({
    description: [
      valueLead(descriptor, values[descriptor.key] ?? descriptor.default),
      settingText(descriptor.key, 'description'),
    ]
      .filter((part) => part !== '')
      .join(' '),
    key: descriptor.key,
    label: settingText(descriptor.key, 'label'),
    page,
    place,
  });
  const core = CORE_SECTIONS.flatMap((kind): readonly SearchEntry[] => {
    const name = t(SECTION_NAMES[kind]);
    const section: SearchEntry = { description: '', key: null, label: name, page: { kind }, place: '' };
    const rows = kind === 'general' ? groups.general : kind === 'scan' ? groups.scan : [];

    return [section, ...rows.map((descriptor) => ofSetting(descriptor, name, { kind }))];
  });
  const modules = groups.modules.flatMap(({ group, items }) =>
    items.flatMap(({ module, rows, switch: enabled }): readonly SearchEntry[] => {
      const name = tText({ args: {}, key: `${module}.enabled.label` });
      const page: SettingsPage = { kind: 'module', module };
      const place = t('ui.screen.settings.place', {
        group: { kind: 'text', value: tText({ args: {}, key: `core.group.${group}.name` }) },
        module: { kind: 'text', value: name },
      });
      const section: SearchEntry = {
        description: '',
        key: null,
        label: name,
        page,
        place: tText({ args: {}, key: `core.group.${group}.name` }),
      };
      const switchEntry: readonly SearchEntry[] =
        enabled === undefined
          ? []
          : [
              {
                description: t('ui.screen.settings.module-switch.description'),
                key: enabled.key,
                label: t('ui.screen.settings.module-switch', { module: { kind: 'text', value: name } }),
                page,
                place,
              },
            ];

      return [section, ...rows.map((row) => ofSetting(row, place, page)), ...switchEntry];
    }),
  );

  return [...core, ...modules];
};

// The hits on one page, for the tree's counts.
export const hitsOn = (hits: readonly SearchHit[], page: SettingsPage): number =>
  hits.filter(({ entry }) => entry.page.kind === page.kind && modulesAgree(entry.page, page)).length;

const modulesAgree = (one: SettingsPage, other: SettingsPage): boolean =>
  one.kind !== 'module' || (other.kind === 'module' && one.module === other.module);
