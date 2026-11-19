import { type JSX, useEffect } from 'react';

import { tText } from '../../i18n';
import { accessOf, notSeen } from '../../state/access';
import { requestSettings } from '../../state/connect';
import { useCoreStore } from '../../state/store';
import { idOf, isOptIn } from './capability-id';
import { CoverageNotice } from './coverage-notice';
import styles from './coverage-notices.module.css';
import { useAccessStatus, useCoverage } from './use-access';

// The notices next to the result a missing permission limits, one per capability the scan could not use, in the
// core's order. Nothing before a scan, for a complete coverage or for a capability not needed on this Mac. A polite
// live region announces a notice that appears after a scan.
export const CoverageNotices = (): JSX.Element => {
  const store = useCoreStore();
  const status = useAccessStatus();
  const coverage = useCoverage();
  const entries = notSeen(coverage).flatMap((entry) => {
    const id = idOf(entry.capability);
    const current = accessOf(status, entry.capability) ?? entry.access;

    return id === null || current === 'not-applicable' ? [] : [{ current, entry, id }];
  });

  const optIn = entries.some(({ entry }) => isOptIn(entry.capability));

  // The opt-ins show whether their switch is on.
  useEffect(() => {
    if (optIn) {
      requestSettings(store);
    }
  }, [optIn, store]);

  return (
    <div aria-live="polite" className={styles.notices}>
      {entries.map(({ current, entry, id }) => (
        <section aria-label={tText({ args: {}, key: `core.access.${id}.name` })} key={id}>
          <CoverageNotice current={current} entry={entry} id={id} />
        </section>
      ))}
    </div>
  );
};
