import type { JSX } from 'react';

import styles from './section-separator.module.css';

export type SectionSeparatorProps = { readonly label: string };

// A heading line between the page's parts: the label, then a rule that fades out. The rule is decoration only.
export const SectionSeparator = ({ label }: SectionSeparatorProps): JSX.Element => (
  <div className={styles.separator}>
    <span className={styles.label}>{label}</span>
    <span aria-hidden="true" className={styles.line} />
  </div>
);
