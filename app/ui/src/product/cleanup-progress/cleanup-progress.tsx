import { type JSX, useId } from 'react';

import type { Method } from '../../ipc/bindings.generated';

import { Button } from '../../design/primitives/button/button';
import { ProgressBar } from '../../design/primitives/progress-bar/progress-bar';
import { t } from '../../i18n';
import styles from './cleanup-progress.module.css';

export type CleanupProgressProps = {
  readonly done: number;
  readonly method: Method;
  // The title of the item in hand; `null` when it is not known.
  readonly now: null | string;
  // Asks the core to stop after the item in hand.
  readonly onStop: () => void;
  // Stop was asked for.
  readonly stopping: boolean;
  readonly total: number;
};

// A cleanup under way: which item of how many, a plain bar, the item in hand, and Stop. Static by the safety rule:
// the numbers change as the core reports, with no counting, glow or stagger, and the bar's fill jumps.
export const CleanupProgress = ({ done, method, now, onStop, stopping, total }: CleanupProgressProps): JSX.Element => {
  const statusId = useId();
  // The item in hand is the one after those done.
  const args = {
    done: { kind: 'count', value: Math.min(done + 1, total) },
    total: { kind: 'count', value: total },
  } as const;

  return (
    <section aria-labelledby={statusId} className={styles.progress}>
      <p className={styles.status} id={statusId}>
        {method === 'trash' ? t('ui.cleanup.moving', args) : t('ui.cleanup.deleting', args)}
      </p>
      <ProgressBar label={t('ui.cleanup.label')} value={total === 0 ? 0 : (done / total) * 100} />
      <p className={styles.now}>{now === null ? null : t('ui.cleanup.now', { title: { kind: 'text', value: now } })}</p>
      <div className={styles.actions}>
        <Button busy={stopping} disabled={stopping} onClick={onStop}>
          {t('ui.cleanup.stop')}
        </Button>
        <p className={styles.stopping} role="status">
          {stopping ? t('ui.cleanup.stopping') : null}
        </p>
      </div>
    </section>
  );
};
