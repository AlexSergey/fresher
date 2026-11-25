import type { JSX } from 'react';

import type { LeftOutCount } from '../../ipc/bindings.generated';

import { Icon } from '../../design/icons/icon';
import { Info } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { ProblemText } from '../problem-text/problem-text';
import styles from './left-out-lines.module.css';

export type LeftOutLinesProps = {
  // Each line as a bar with an `i` icon, as a category's details show them.
  readonly bar?: boolean;
  // The engine's counts for one category: one per reason.
  readonly counts: readonly LeftOutCount[];
};

// The line over one count, in the core's words; the translator picks the plural form by the count.
const summaryOf = ({ items, sizes }: LeftOutCount): string =>
  tText({
    args: { count: { kind: 'count', value: items }, size: { kind: 'size', value: sizes.allocated } },
    key: 'core.left-out.summary',
  });

// What a category recognized and did not offer: how many items, how big, and why, one quiet line per reason.
// Nothing is counted here: the numbers and both texts come from the core.
export const LeftOutLines = ({ bar = false, counts }: LeftOutLinesProps): JSX.Element | null =>
  counts.length === 0 ? null : (
    <ul aria-label={t('ui.left-out-lines.label')} className={styles.lines} data-bar={bar ? true : undefined}>
      {counts.map((count) => (
        <li className={styles.line} key={count.reason.key}>
          {bar ? <Icon className={styles.icon ?? ''} icon={Info} /> : null}
          <span>{summaryOf(count)} </span>
          <ProblemText text={count.reason} />
        </li>
      ))}
    </ul>
  );
