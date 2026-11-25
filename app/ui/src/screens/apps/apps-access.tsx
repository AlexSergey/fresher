import type { JSX } from 'react';

import type { Capability } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { tText } from '../../i18n';
import { accessOf, notSeen } from '../../state/access';
import { idOf } from '../permission/capability-id';
import { useAccessStatus, useCoverage } from '../permission/use-access';

// The permissions that change what Apps shows: the folders of an app's footprint (its containers, its data in the
// Library) are read only with them. The guarded folders of the user's files do not hold an app's footprint.
const SHAPES_APPS: ReadonlySet<Capability['kind']> = new Set(['all-files', 'other-apps-data']);

// One short notice per permission the last scan lacked that leaves an app's footprint incomplete, with the core's
// sentence of what was not seen; nothing for the folders that do not hold an app's files, and nothing once allowed.
// The Scan screen keeps the full notices with their actions.
export const AppsAccess = (): JSX.Element | null => {
  const status = useAccessStatus();
  const lacking = notSeen(useCoverage()).flatMap(({ access, capability }) => {
    const id = idOf(capability);
    const current = accessOf(status, capability) ?? access;

    return id === null || !SHAPES_APPS.has(capability.kind) || current === 'granted' || current === 'not-applicable'
      ? []
      : [id];
  });

  return lacking.length === 0 ? null : (
    <>
      {lacking.map((id) => (
        <Notice key={id} title={tText({ args: {}, key: `core.access.${id}.name` })} tone="info">
          <p>{tText({ args: {}, key: `core.access.${id}.missing` })}</p>
        </Notice>
      ))}
    </>
  );
};
