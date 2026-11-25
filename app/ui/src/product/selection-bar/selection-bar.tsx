import { type JSX, useEffect, useEffectEvent, useState } from 'react';

import type { TextArgs } from '../../i18n/types';
import type { DangerLevel, Sizes } from '../../ipc/bindings.generated';

import { useAnnounce } from '../../design/feedback/announcer/announcer';
import { useAnimatedNumber, usePresence } from '../../design/motion';
import { Button } from '../../design/primitives/button/button';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { DURATION } from '../../design/tokens/motion';
import { t } from '../../i18n';
import { formatSize } from '../../i18n/format-size';
import { honestSize } from '../sizes';
import styles from './selection-bar.module.css';

export type SelectionBarProps = {
  readonly onClear: () => void;
  readonly onReview: () => void;
  // `null`, or a count of 0 with nothing to check, when nothing is selected: the bar is absent.
  readonly summary: null | Summary;
};

// What the selection adds up to (the store's `SelectionSummary`): a preview, not the confirmation.
type Summary = {
  readonly count: number;
  readonly highest: DangerLevel | null;
  readonly sizes: Sizes;
  // The "Look before removing" categories brought to Review, none of whose items goes until ticked there.
  readonly toCheck: { readonly categories: number };
};

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

const shownOf = (summary: null | Summary): null | Summary =>
  summary === null || (summary.count === 0 && summary.toCheck.categories === 0) ? null : summary;

const same = (a: null | Summary, b: Summary): boolean =>
  a !== null &&
  a.count === b.count &&
  a.toCheck.categories === b.toCheck.categories &&
  a.sizes.allocated === b.sizes.allocated &&
  a.sizes.completeness === b.sizes.completeness;

// The total, counting quickly (200 ms) toward a new sum; assistive technology reads only its final words.
const Total = ({ sizes }: { readonly sizes: Sizes }): JSX.Element => {
  const bytes = useAnimatedNumber(sizes.allocated, { duration: DURATION.base });

  return (
    <Tooltip content={t('ui.selection-bar.total.hint')}>
      {(describedBy) => (
        // eslint-disable-next-line jsx-a11y-x/no-noninteractive-tabindex -- the tooltip needs a focusable anchor
        <span aria-describedby={describedBy} className={styles.total} data-numeric tabIndex={0}>
          <span aria-hidden="true">{formatSize(bytes, sizes.completeness)}</span>
          <VisuallyHidden>{honestSize(sizes).accessibleText}</VisuallyHidden>
        </span>
      )}
    </Tooltip>
  );
};

const toCheckCount = (summary: Summary): TextArgs => ({
  count: { kind: 'count', value: summary.toCheck.categories },
});

// What is said when the bar appears: what goes and its size, then how many categories are to check; only that while
// nothing goes.
const announcementOf = (summary: Summary): string => {
  if (summary.count === 0) {
    return t('ui.selection-bar.only-to-check', toCheckCount(summary));
  }

  const goes = t('ui.selection-bar.announcement', {
    count: { kind: 'count', value: summary.count },
    size: { kind: 'text', value: honestSize(summary.sizes).accessibleText },
  });

  return summary.toCheck.categories === 0 ? goes : `${goes} ${t('ui.selection-bar.to-check', toCheckCount(summary))}`;
};

// The bar that floats at the bottom centre of a screen while something is selected: the count, the total, how many
// categories are to check in Review, Clear and Review; while nothing goes, only how many are to check. It rises and
// fades in, is announced as it appears, and fades out when the selection empties.
export const SelectionBar = ({ onClear, onReview, summary }: SelectionBarProps): JSX.Element | null => {
  const shown = shownOf(summary);
  const open = shown !== null;
  const announce = useAnnounce();
  const { mounted, ref } = usePresence<HTMLDivElement>(open, { duration: DURATION.fast, exit: FADE_OUT });
  // What the bar showed last: kept while it fades out.
  const [kept, setKept] = useState(shown);

  if (shown !== null && !same(kept, shown)) {
    setKept(shown);
  }

  const announceShown = useEffectEvent((): void => {
    if (shown !== null) {
      announce(announcementOf(shown));
    }
  });

  // Announced when it appears, and when the categories to check change: a Look category brought to Review (or taken
  // back) changes nothing that goes, so the count alone would not tell it.
  const toCheck = shown?.toCheck.categories ?? 0;

  useEffect(() => {
    if (open) {
      announceShown();
    }
  }, [open, toCheck]);

  if (!mounted || kept === null) {
    return null;
  }

  return (
    <div aria-label={t('ui.selection-bar.label')} className={styles.anchor} role="region">
      <div className={styles.rising} ref={ref}>
        {kept.count === 0 ? (
          <span className={styles.toCheck}>{t('ui.selection-bar.only-to-check', toCheckCount(kept))}</span>
        ) : (
          <>
            <span className={styles.count}>
              {t('ui.selection-bar.count', { count: { kind: 'count', value: kept.count } })}
            </span>
            <Total sizes={kept.sizes} />
            {kept.toCheck.categories > 0 ? (
              <span className={styles.toCheck}>{t('ui.selection-bar.to-check', toCheckCount(kept))}</span>
            ) : null}
          </>
        )}
        <span className={styles.actions}>
          <Button onClick={onClear} variant="ghost">
            {t('ui.selection-bar.clear')}
          </Button>
          <Button onClick={onReview} variant="primary">
            {t('ui.selection-bar.review')}
          </Button>
        </span>
      </div>
    </div>
  );
};
