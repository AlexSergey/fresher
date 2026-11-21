import { type JSX, useId } from 'react';

import type { CategoryId } from '../../ipc/bindings.generated';

import { t, tText } from '../../i18n';
import styles from './category-screen.module.css';

const textOf = (key: string): string => tText({ args: {}, key });

// What the category is, in the core's four plain sentences, shown open.
export const CategoryAbout = ({ category }: { readonly category: CategoryId }): JSX.Element => {
  const headingId = useId();

  return (
    <section aria-labelledby={headingId} className={styles.part}>
      <h2 className={styles.heading} id={headingId}>
        {t('ui.screen.category.about')}
      </h2>
      <dl className={styles.sentences}>
        <dt>{t('ui.screen.category.what')}</dt>
        <dd>{textOf(`${category}.what`)}</dd>
        <dt>{t('ui.screen.category.made-by')}</dt>
        <dd>{textOf(`${category}.made-by`)}</dd>
        <dt>{t('ui.screen.category.if-removed')}</dt>
        <dd>{textOf(`${category}.if-removed`)}</dd>
        <dt>{t('ui.screen.category.comes-back')}</dt>
        <dd>{textOf(`${category}.comes-back`)}</dd>
      </dl>
    </section>
  );
};
