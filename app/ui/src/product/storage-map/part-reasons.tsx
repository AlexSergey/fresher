import type { JSX } from 'react';

import type { RemainderReason } from '../../ipc/bindings.generated';

import { t } from '../../i18n';
import { reasonText } from './parts';
import styles from './storage-map.module.css';

const Reason = ({ reason }: { readonly reason: RemainderReason }): JSX.Element => (
  <li>
    {reasonText(reason)}
    {reason.kind === 'unreadable' ? (
      <>
        {' '}
        {t('ui.storage-map.unreadable', { count: { kind: 'count', value: reason.folders } })}
        {reason.needs?.kind === 'all-files' ? <> {t('ui.storage-map.needs.all-files')}</> : null}
      </>
    ) : null}
  </li>
);

// The remainder's reasons, plain lines (never a size each): what kept the scan from measuring it.
export const PartReasons = ({ reasons }: { readonly reasons: readonly RemainderReason[] }): JSX.Element => (
  <ul className={styles.reasons}>
    {reasons.map((reason) => (
      <Reason key={reason.kind} reason={reason} />
    ))}
  </ul>
);
