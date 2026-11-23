import { type JSX, type ReactNode, useId } from 'react';

import type { ReviewPart } from './review-groups';

import { tText } from '../../i18n';
import styles from './review-screen.module.css';

export type ReviewPartSectionProps = {
  readonly children: ReactNode;
  readonly part: ReviewPart;
};

// One of Review's two parts: its name and meaning, both the core's, over its zones. The danger part is set apart by a
// border in its highest level's colour; the safe part has none.
export const ReviewPartSection = ({ children, part }: ReviewPartSectionProps): JSX.Element => {
  const headingId = useId();

  return (
    <section aria-labelledby={headingId} className={styles.part} data-level={part.highest} data-part={part.part}>
      <div className={styles.partHead}>
        <h2 className={styles.partName} id={headingId}>
          {tText({ args: {}, key: `core.danger.part.${part.part}.name` })}
        </h2>
        <p className={styles.partMeaning}>{tText({ args: {}, key: `core.danger.part.${part.part}.meaning` })}</p>
      </div>
      {children}
    </section>
  );
};
