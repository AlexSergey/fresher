import { type JSX, useId } from 'react';

import type { CategoryId } from '../../ipc/bindings.generated';

import { t, tText } from '../../i18n';
import styles from './category-about.module.css';

const textOf = (key: string): string => tText({ args: {}, key });

// The three answers about a category, side by side: what it is (and who makes it), what happens if it is removed,
// and whether it comes back. The words are the core's.
export const CategoryAbout = ({ category }: { readonly category: CategoryId }): JSX.Element => {
  const whatId = useId();
  const removedId = useId();
  const backId = useId();

  return (
    <div className={styles.answers}>
      <section aria-labelledby={whatId} className={styles.answer}>
        <h2 className={styles.title} id={whatId}>
          {t('ui.screen.category.what')}
        </h2>
        <p className={styles.text}>{textOf(`${category}.what`)}</p>
        <p className={styles.madeBy}>{textOf(`${category}.made-by`)}</p>
      </section>
      <section aria-labelledby={removedId} className={styles.answer}>
        <h2 className={styles.title} id={removedId}>
          {t('ui.screen.category.if-removed')}
        </h2>
        <p className={styles.text}>{textOf(`${category}.if-removed`)}</p>
      </section>
      <section aria-labelledby={backId} className={styles.answer}>
        <h2 className={styles.title} id={backId}>
          {t('ui.screen.category.comes-back')}
        </h2>
        <p className={styles.text}>{textOf(`${category}.comes-back`)}</p>
      </section>
    </div>
  );
};
