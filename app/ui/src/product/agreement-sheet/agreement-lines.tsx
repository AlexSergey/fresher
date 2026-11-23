import type { JSX } from 'react';

import type { AgreementView } from '../../ipc/bindings.generated';

import { tText } from '../../i18n';
import styles from './agreement-sheet.module.css';

// The agreement's five lines, in the order the core lists them: the sheet and Settings show the same list.
export const AgreementLines = ({ view }: { readonly view: AgreementView }): JSX.Element => (
  <ul className={styles.lines}>
    {view.lines.map((line) => (
      <li key={line.key}>{tText(line)}</li>
    ))}
  </ul>
);
