import type { SkipCode, SkipReason } from '../ipc/bindings.generated';

import { tText, tTextIfKnown } from '../i18n';

const joined = (what: string, how: null | string): string => (how === null ? what : `${what} ${how}`);

const howOf = (reason: SkipReason): null | string => {
  if (reason.kind === 'in-use') {
    return tTextIfKnown({ args: { app: { kind: 'text', value: reason.app } }, key: 'core.cleanup.skip.in-use.how' });
  }

  return reason.kind === 'not-removable'
    ? tTextIfKnown({ args: {}, key: `core.removability.${reason.removability}.how` })
    : tTextIfKnown({ args: {}, key: `core.cleanup.skip.${reason.kind}.how` });
};

// Why an item was left out or skipped, and what the user can do about it where the core says: the core's texts.
export const skipReasonText = (reason: SkipReason): string =>
  joined(tText({ args: {}, key: `core.cleanup.skip.${reason.kind}` }), howOf(reason));

// The same from the code the history keeps; an instruction that names an app is not kept, so it is not shown.
export const skipCodeText = (code: SkipCode): string =>
  joined(
    tText({ args: {}, key: `core.cleanup.skip.${code}` }),
    code === 'in-use' ? null : tTextIfKnown({ args: {}, key: `core.cleanup.skip.${code}.how` }),
  );
