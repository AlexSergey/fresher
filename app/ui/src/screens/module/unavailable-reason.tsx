import type { JSX } from 'react';

import type { Capability } from '../../ipc/bindings.generated';

import { accessOf } from '../../state/access';
import { idOf } from '../permission/capability-id';
import { missingSentence, notApplicable } from '../permission/category-access';
import { CoverageNotice } from '../permission/coverage-notice';
import { useAccessStatus } from '../permission/use-access';

export type UnavailableReasonProps = {
  readonly id: string;
  readonly missing: readonly Capability[];
};

// Why a category cannot be cleaned now. A tool says its state's note in one sentence; an OS permission or an opt-in
// shows the same notice, with the same action, as the overview's coverage notice.
export const UnavailableReason = ({ id, missing }: UnavailableReasonProps): JSX.Element => {
  const status = useAccessStatus();

  return (
    <div id={id}>
      {missing.map((capability) => {
        const capabilityId = idOf(capability);
        const key = capabilityId ?? JSON.stringify(capability);

        if (capabilityId === null) {
          return <p key={key}>{missingSentence(capability, status)}</p>;
        }

        return notApplicable(status, capability) ? null : (
          <CoverageNotice
            current={accessOf(status, capability)}
            entry={{ access: null, capability, folders: 0 }}
            id={capabilityId}
            key={key}
          />
        );
      })}
    </div>
  );
};
