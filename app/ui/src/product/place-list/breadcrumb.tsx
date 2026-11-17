import type { JSX } from 'react';

import { t } from '../../i18n';
import styles from './place-list.module.css';

export type BreadcrumbProps = {
  // Outermost first; the last is the folder open.
  readonly crumbs: readonly Crumb[];
  // Opens the step at `index`, as it is: the view does not go down again from it.
  readonly onStep: (index: number) => void;
};

// One step of the way down: a folder, or the volume's scanned folders when it holds several.
export type Crumb = { readonly key: string; readonly name: string };

// The way down to the open folder, "Library › Containers › com.example.app": every step but the last is a button back
// to it; the last is where the view is.
export const Breadcrumb = ({ crumbs, onStep }: BreadcrumbProps): JSX.Element => (
  <nav aria-label={t('ui.place-list.breadcrumb')} className={styles.breadcrumb}>
    <ol className={styles.crumbs}>
      {crumbs.map(({ key, name }, index) => (
        <li className={styles.crumb} key={key}>
          {index > 0 ? (
            <span aria-hidden="true" className={styles.separator}>
              ›
            </span>
          ) : null}
          {index === crumbs.length - 1 ? (
            <span aria-current="location" className={styles.current}>
              {name}
            </span>
          ) : (
            <button
              aria-label={t('ui.place-list.back', { folder: { kind: 'text', value: name } })}
              className={styles.step}
              onClick={() => {
                onStep(index);
              }}
              type="button"
            >
              {name}
            </button>
          )}
        </li>
      ))}
    </ol>
  </nav>
);
