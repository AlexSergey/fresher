import { type JSX, useEffect, useEffectEvent, useState } from 'react';

import type { DangerLevel, Sizes } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { useAnnounce } from '../../design/feedback/announcer/announcer';
import { Surface } from '../../design/layout/surface/surface';
import { useAnimatedNumber, usePresence } from '../../design/motion';
import { Button } from '../../design/primitives/button/button';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { DURATION } from '../../design/tokens/motion';
import { t } from '../../i18n';
import { formatSize } from '../../i18n/format-size';
import { levelName } from '../finding-row/texts';
import { honestSize } from '../sizes';
import styles from './selection-bar.module.css';

export type SelectionBarProps = {
  readonly onClear: () => void;
  readonly onReview: () => void;
  // `null`, or a count of 0, when nothing is selected: the bar is absent.
  readonly summary: null | Summary;
};

type Shown = Summary & { readonly highest: DangerLevel };

// What the selection adds up to (the store's `SelectionSummary`): a preview, not the confirmation.
type Summary = {
  readonly count: number;
  readonly highest: DangerLevel | null;
  readonly sizes: Sizes;
};

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

const shownOf = (summary: null | Summary): null | Shown =>
  summary === null || summary.count === 0 || summary.highest === null ? null : { ...summary, highest: summary.highest };

const same = (a: null | Shown, b: Shown): boolean =>
  a !== null &&
  a.count === b.count &&
  a.highest === b.highest &&
  a.sizes.allocated === b.sizes.allocated &&
  a.sizes.completeness === b.sizes.completeness;

// The total, counting quickly (200 ms) toward a new sum; assistive technology reads only its final words.
const Total = ({ sizes }: { readonly sizes: Sizes }): JSX.Element => {
  const bytes = useAnimatedNumber(sizes.allocated, { duration: DURATION.base });

  return (
    <span className={styles.total} data-numeric>
      <span aria-hidden="true">{formatSize(bytes, sizes.completeness)}</span>
      <VisuallyHidden>{honestSize(sizes).accessibleText}</VisuallyHidden>
    </span>
  );
};

// The bar that floats at the bottom centre of a screen while something is selected: the count, the total, the highest
// danger level, Clear and Review. It rises and fades in, is announced once, and fades out when the selection empties.
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
      announce(
        t('ui.selection-bar.announcement', {
          count: { kind: 'count', value: shown.count },
          size: { kind: 'text', value: honestSize(shown.sizes).accessibleText },
        }),
      );
    }
  });

  useEffect(() => {
    if (open) {
      announceShown();
    }
  }, [open]);

  if (!mounted || kept === null) {
    return null;
  }

  return (
    <div aria-label={t('ui.selection-bar.label')} className={styles.anchor} role="region">
      <Surface className={styles.bar ?? ''} elevation={3} padding="none" radius="lg" ref={ref}>
        <span className={styles.count}>
          {t('ui.selection-bar.count', { count: { kind: 'count', value: kept.count } })}
        </span>
        <Total sizes={kept.sizes} />
        <span className={styles.level}>
          <VisuallyHidden>{t('ui.selection-bar.highest')}</VisuallyHidden>
          <DangerBadge label={levelName(kept.highest)} level={kept.highest} />
        </span>
        <span className={styles.actions}>
          <Button onClick={onClear} variant="ghost">
            {t('ui.selection-bar.clear')}
          </Button>
          <Button onClick={onReview} variant="primary">
            {t('ui.selection-bar.review')}
          </Button>
        </span>
      </Surface>
    </div>
  );
};
