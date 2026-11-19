import type { DangerLevel, Removability } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';

// The texts a finding's values are shown with: the danger level's word and sentence from the core's
// dictionary. Sizes are shown through `honestSize` of `../sizes`.

export const levelName = (level: DangerLevel): string => tText({ args: {}, key: `core.danger.${level}.name` });

export const levelMeaning = (level: DangerLevel): string => tText({ args: {}, key: `core.danger.${level}.meaning` });

// Why the app may not remove a finding, from the core's dictionary: its short word, and the sentence that says what
// the user can do (or, for macOS's own items, why nothing is offered).
export const removabilityName = (removability: Removability): string =>
  tText({ args: {}, key: `core.removability.${removability}.name` });

export const removabilityHow = (removability: Exclude<Removability, 'removable'>): string =>
  tText({ args: {}, key: `core.removability.${removability}.how` });
