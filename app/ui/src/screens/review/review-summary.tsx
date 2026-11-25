import type { JSX } from 'react';

import type { DangerLevel, Sizes } from '../../ipc/bindings.generated';
import type { ReviewLevel } from './review-groups';

import { SizeText } from '../../design/data/size-text/size-text';
import { t, tText } from '../../i18n';
import { BY_HAND } from '../../product/danger-zone/zones';
import { honestSize } from '../../product/sizes';
import styles from './review-summary.module.css';

export type ReviewSummaryProps = {
  readonly count: number;
  readonly levels: readonly ReviewLevel[];
  readonly sizes: Sizes;
};

const zoneOf = (level: DangerLevel): string => tText({ args: {}, key: `core.danger.${level}.zone` });

// The bar's accessible name: each group with its size, in the bar's order.
const barLabel = (levels: readonly ReviewLevel[]): string =>
  t('ui.screen.review.bar', {
    parts: {
      kind: 'text',
      value: levels
        .map(({ level, sizes }) =>
          t('ui.screen.review.bar.part', {
            group: { kind: 'text', value: zoneOf(level) },
            size: { kind: 'text', value: honestSize(sizes).accessibleText },
          }),
        )
        .join('; '),
    },
  });

// One group's chip: its colour, its name and its size; a "Look before removing" group says how many of its items are
// not checked yet.
const Chip = ({ level, sizes, unchecked }: ReviewLevel): JSX.Element => (
  <span className={styles.chip} data-level={level}>
    <span aria-hidden="true" className={styles.swatch} />
    {zoneOf(level)}
    <b className={styles.chipSize}>
      <SizeText {...honestSize(sizes)} />
    </b>
    {BY_HAND.has(level) && unchecked > 0 ? (
      <span className={styles.notChecked}>
        {t('ui.screen.review.not-checked', { count: { kind: 'count', value: unchecked } })}
      </span>
    ) : null}
  </span>
);

// What goes to the Trash, summed with the store's rule (the confirmation shows only the core's plan): the size and the
// count, that it can be put back, a bar split by the size of each group present and a chip for each group.
export const ReviewSummary = ({ count, levels, sizes }: ReviewSummaryProps): JSX.Element => (
  <section aria-label={t('ui.screen.review.summary')} className={styles.summary}>
    <div className={styles.top}>
      <div className={styles.figures}>
        <span className={styles.size}>
          <SizeText {...honestSize(sizes)} />
        </span>
        <span className={styles.goes}>
          {t('ui.screen.review.goes', {
            items: { kind: 'text', value: t('ui.screen.review.count', { count: { kind: 'count', value: count } }) },
          })}
        </span>
      </div>
      <span className={styles.putBack}>{t('ui.screen.review.put-back')}</span>
    </div>
    <div aria-label={barLabel(levels)} className={styles.bar} role="img">
      {levels.map(({ level, sizes: part }) => (
        <span className={styles.segment} data-level={level} key={level} style={{ flexGrow: part.allocated }} />
      ))}
    </div>
    <div className={styles.chips}>
      {levels.map((level) => (
        <Chip key={level.level} {...level} />
      ))}
    </div>
  </section>
);
