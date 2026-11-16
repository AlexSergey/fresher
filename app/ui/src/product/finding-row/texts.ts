import type { DangerLevel } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';

// The texts a finding's values are shown with: the danger level's word and sentence from the core's
// dictionary. Sizes are shown through `honestSize` of `../sizes`.

export const levelName = (level: DangerLevel): string => tText({ args: {}, key: `core.danger.${level}.name` });

export const levelMeaning = (level: DangerLevel): string => tText({ args: {}, key: `core.danger.${level}.meaning` });
