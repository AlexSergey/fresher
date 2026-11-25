import type { JSX } from 'react';

import type { LeftOutCount, LeftOutEntry } from '../../ipc/bindings.generated';

import { PathText } from '../../design/data/path-text/path-text';
import { SizeText } from '../../design/data/size-text/size-text';
import { Icon } from '../../design/icons/icon';
import { Info } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { shortPath } from '../paths/short-path';
import { ProblemText } from '../problem-text/problem-text';
import { honestSize } from '../sizes';
import styles from './left-out-lines.module.css';

export type LeftOutLinesProps = {
  // Each line held back for now as a bar with an `i` icon, as a category's details show them.
  readonly bar?: boolean;
  // The engine's counts for one category: one per reason.
  readonly counts: readonly LeftOutCount[];
  // The home folder the core reports: each item's path shows from `~`.
  readonly home?: null | string;
};

// The line over one count, in the core's words; the translator picks the plural form by the count. A count kept for
// good says "kept", one held back for now "left out".
const summaryOf = ({ items, kind, sizes }: LeftOutCount): string =>
  tText({
    args: { count: { kind: 'count', value: items }, size: { kind: 'size', value: sizes.allocated } },
    key: kind === 'kept' ? 'core.left-out.kept' : 'core.left-out.summary',
  });

// One item held back: its name over its path from `~` (the full path in the tooltip), and its size.
const Item = ({ entry, home }: { readonly entry: LeftOutEntry; readonly home: null | string }): JSX.Element => (
  <li className={styles.item}>
    <span className={styles.names}>
      <span className={styles.title}>{tText(entry.title)}</span>
      <PathText display={shortPath(entry.path, home)} path={entry.path} />
    </span>
    <span className={styles.size}>
      <SizeText {...honestSize(entry.sizes)} />
    </span>
  </li>
);

// The items of a count held back for now: the largest the core kept, then how many more it only counted.
const Items = ({ count, home }: { readonly count: LeftOutCount; readonly home: null | string }): JSX.Element | null => {
  const more = count.items - count.shown.length;

  return count.shown.length === 0 && more <= 0 ? null : (
    <ul aria-label={t('ui.left-out-lines.items')} className={styles.items}>
      {count.shown.map((entry) => (
        <Item entry={entry} home={home} key={entry.path} />
      ))}
      {more > 0 ? (
        <li className={styles.more}>{t('ui.left-out-lines.more', { count: { kind: 'count', value: more } })}</li>
      ) : null}
    </ul>
  );
};

// The counts held back for now first, then the ones kept for good, each part in the core's order.
const ordered = (counts: readonly LeftOutCount[]): readonly LeftOutCount[] => [
  ...counts.filter(({ kind }) => kind === 'for-now'),
  ...counts.filter(({ kind }) => kind === 'kept'),
];

// What a category recognized and did not offer, one line per reason: how many items, how big and why, then each
// item held back for now by name, path and size. A count kept for good (the version you use) is one quiet line with
// no item, after the others. Nothing is counted here but the items the core did not list: the numbers and the texts
// come from the core.
export const LeftOutLines = ({ bar = false, counts, home = null }: LeftOutLinesProps): JSX.Element | null =>
  counts.length === 0 ? null : (
    <ul aria-label={t('ui.left-out-lines.label')} className={styles.lines} data-bar={bar ? true : undefined}>
      {ordered(counts).map((count) => (
        <li className={styles.line} data-kind={count.kind} key={`${count.kind}:${count.reason.key}`}>
          <span className={styles.summary}>
            {bar && count.kind === 'for-now' ? <Icon className={styles.icon ?? ''} icon={Info} /> : null}
            <span>{summaryOf(count)} </span>
            <ProblemText text={count.reason} />
          </span>
          {count.kind === 'for-now' ? <Items count={count} home={home} /> : null}
        </li>
      ))}
    </ul>
  );
