import type { SettingProblem, Unit } from '../../ipc/bindings.generated';

import { t, tText } from '../../i18n';

const count = (value: number): { readonly kind: 'count'; readonly value: number } => ({ kind: 'count', value });

// What a stored value's problem means to the user; the setting keeps its default.
export const problemText = (problem: SettingProblem): string => {
  switch (problem.kind) {
    case 'out-of-range':
      return t('ui.settings-row.problem.out-of-range', { max: count(problem.max), min: count(problem.min) });
    case 'refused-folder':
      return t('ui.settings-row.problem.refused-folder', { folder: { kind: 'path', value: problem.folder } });
    case 'refused-id':
      return t('ui.settings-row.problem.refused-id', { id: { kind: 'text', value: problem.id } });
    case 'unknown-choice':
      return t('ui.settings-row.problem.unknown-choice');
    case 'unknown-key':
      return t('ui.settings-row.problem.unknown-key');
    case 'wrong-type':
      return t('ui.settings-row.problem.wrong-type');
  }
};

// The limits of a number, with its unit when it has one: "From 1 to 365 days".
export const limitsText = (min: number, max: number, unit: null | Unit): string => {
  const args = { max: count(max), min: count(min) };

  switch (unit) {
    case 'bytes':
      return t('ui.settings-row.limits.bytes', args);
    case 'days':
      return t('ui.settings-row.limits.days', args);
    case null:
      return t('ui.settings-row.limits', args);
  }
};

export const notInLimitsText = (min: number, max: number): string =>
  t('ui.settings-row.not-in-limits', { max: count(max), min: count(min) });

// A choice's label, `<key>.options.<option>` in the core's dictionary; an option it does not name shows as written.
export const optionText = (key: string, option: string): string => {
  const optionKey = `${key}.options.${option}`;
  const text = tText({ args: {}, key: optionKey });

  return text === optionKey ? option : text;
};

// A setting's own texts, found by convention: `<key>.label` and `<key>.description`.
export const settingText = (key: string, part: 'description' | 'label'): string =>
  tText({ args: {}, key: `${key}.${part}` });

// The unit written after a number's field and in its slider's name; a number without one has none.
export const unitText = (unit: null | Unit): string | undefined => {
  switch (unit) {
    case 'bytes':
      return t('ui.settings-row.unit.bytes');
    case 'days':
      return t('ui.settings-row.unit.days');
    case null:
      return undefined;
  }
};
