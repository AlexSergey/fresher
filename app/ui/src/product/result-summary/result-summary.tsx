import { type JSX, useEffect, useId, useState } from 'react';

import type { Sizes, Text } from '../../ipc/bindings.generated';

import { AnimatedNumber } from '../../design/data/animated-number/animated-number';
import { AmbientLight } from '../../design/effects/ambient-light/ambient-light';
import { Icon } from '../../design/icons/icon';
import { Check } from '../../design/icons/icons';
import { useMotion } from '../../design/motion';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { formatSize } from '../../i18n/format-size';
import styles from './result-summary.module.css';

export type ResultSummaryProps = {
  readonly freed: Sizes;
  readonly method: 'permanent' | 'trash';
  readonly moved: number;
  readonly onDone: () => void;
  // Shown after a move to the Trash only.
  readonly onOpenTrash?: () => void;
  readonly skipped: readonly SkippedItem[];
};

type SkippedItem = { readonly reason: Text; readonly title: Text };

// The cleaned moment: a check mark that draws itself, the freed size counting up from 0 over one soft green light
// wave, then — with no motion — what was moved, what was skipped and why, and the way out. Serious, not festive: no
// bounce, no confetti. Under reduced motion the final values fade in once; with motion off they are simply there.
export const ResultSummary = ({
  freed,
  method,
  moved,
  onDone,
  onOpenTrash,
  skipped,
}: ResultSummaryProps): JSX.Element => {
  const motion = useMotion();
  const titleId = useId();
  const skippedId = useId();
  // The number counts from 0: it mounts at 0 and is given the freed size on the next frame.
  const [counted, setCounted] = useState(0);
  // The wave and the green light belong to the first moment only; once the wave ends they never come back.
  const [waved, setWaved] = useState(false);
  const waving = motion === 'full' && !waved;

  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      setCounted(freed.allocated);
    });

    return (): void => {
      cancelAnimationFrame(frame);
    };
  }, [freed.allocated]);

  const count = { count: { kind: 'count', value: moved } } as const;

  return (
    <section aria-labelledby={titleId} className={styles.summary} data-motion={motion}>
      <AmbientLight tone={waving ? 'safe' : 'iris'} />
      <div className={styles.moment}>
        <span className={styles.check}>
          <Icon icon={Check} size={24} />
        </span>
        <h2 className={styles.title} id={titleId}>
          {t('ui.result.freed')}
        </h2>
        <span className={styles.freed}>
          {waving ? (
            <span
              aria-hidden="true"
              className={styles.wave}
              data-light-wave
              onAnimationEnd={() => {
                setWaved(true);
              }}
            />
          ) : null}
          <span className={styles.number}>
            <AnimatedNumber
              duration="deliberate"
              format={(value) => formatSize(value, freed.completeness)}
              value={motion === 'full' ? counted : freed.allocated}
            />
          </span>
        </span>
      </div>
      <p className={styles.method}>
        {method === 'trash' ? t('ui.result.moved', count) : t('ui.result.deleted', count)}
      </p>
      {skipped.length === 0 ? null : (
        <div className={styles.skipped}>
          <p className={styles.skippedTitle} id={skippedId}>
            {t('ui.result.skipped', { count: { kind: 'count', value: skipped.length } })}
          </p>
          <ul aria-labelledby={skippedId} className={styles.list}>
            {skipped.map((item, index) => (
              // Two skipped items may carry the same texts; their order is the order the core reported them in.
              // eslint-disable-next-line @eslint-react/no-array-index-key -- the list is fixed once shown
              <li className={styles.item} key={index}>
                <span className={styles.itemTitle}>{tText(item.title)}</span>
                <span className={styles.reason}>{tText(item.reason)}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className={styles.actions}>
        {method === 'trash' && onOpenTrash !== undefined ? (
          <Button onClick={onOpenTrash}>{t('ui.result.open-trash')}</Button>
        ) : null}
        <Button onClick={onDone} variant="primary">
          {t('ui.result.done')}
        </Button>
      </div>
    </section>
  );
};
