import { type JSX, type RefObject, useId } from 'react';

import type { Sizes } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import bar from '../../product/selection-bar/selection-bar.module.css';
import { honestSize } from '../../product/sizes';
import styles from './review-bar.module.css';

export type ReviewBarProps = {
  // The core is checking each item before it shows its plan.
  readonly checking: boolean;
  readonly count: number;
  readonly onClear: () => void;
  readonly onRemove: () => void;
  // The confirmation and a refusal give the focus back to the button: a click does not focus a button in WebKit.
  readonly removeRef: RefObject<HTMLButtonElement | null>;
  readonly sizes: Sizes;
};

// The bar that floats at the bottom of Review: how many items go and their size, Clear, and Move to the Trash, which
// asks the core to check each item, then opens the confirmation. While nothing goes it says to tick an item first, and
// the button is inactive.
export const ReviewBar = ({ checking, count, onClear, onRemove, removeRef, sizes }: ReviewBarProps): JSX.Element => {
  const noneId = useId();
  const none = count === 0;

  return (
    <div aria-label={t('ui.screen.review.bar-label')} className={bar.anchor} role="region">
      <div className={bar.bar}>
        {none ? (
          <span className={styles.count} id={noneId}>
            {t('ui.screen.review.remove.none')}
          </span>
        ) : (
          <>
            <span className={styles.count}>
              {t('ui.screen.review.count', { count: { kind: 'count', value: count } })}
            </span>
            <span className={styles.total}>
              <SizeText {...honestSize(sizes)} />
            </span>
          </>
        )}
        {/* Out of the flow (absolute), so it takes no gap of the bar. */}
        <span className="u-visually-hidden" role="status">
          {checking ? t('ui.screen.review.checking') : null}
        </span>
        <span className={bar.actions}>
          <Button onClick={onClear} variant="ghost">
            {t('ui.screen.review.clear')}
          </Button>
          <Button
            aria-describedby={none ? noneId : undefined}
            busy={checking}
            disabled={none}
            onClick={onRemove}
            ref={removeRef}
            variant="primary"
          >
            {t('ui.screen.review.remove')}
          </Button>
        </span>
      </div>
    </div>
  );
};
