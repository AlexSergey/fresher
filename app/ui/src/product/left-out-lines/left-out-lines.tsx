import { type JSX, useId, useState } from 'react';

import type { LeftOutCount, LeftOutEntry } from '../../ipc/bindings.generated';

import { PathText } from '../../design/data/path-text/path-text';
import { SizeText } from '../../design/data/size-text/size-text';
import { Icon } from '../../design/icons/icon';
import { ChevronRight, Info } from '../../design/icons/icons';
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
  // Whether each line's items start shown; folded by default.
  readonly open?: boolean;
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

// How many items the core counted but did not list.
const moreOf = (count: LeftOutCount): number => count.items - count.shown.length;

// The items of a count held back for now: the largest the core kept, then how many more it only counted. Hidden while
// the line is folded.
const Items = ({
  count,
  hidden,
  home,
  id,
}: {
  readonly count: LeftOutCount;
  readonly hidden: boolean;
  readonly home: null | string;
  readonly id: string;
}): JSX.Element => {
  const more = moreOf(count);

  return (
    <ul aria-label={t('ui.left-out-lines.items')} className={styles.items} hidden={hidden} id={id}>
      {count.shown.map((entry) => (
        <Item entry={entry} home={home} key={entry.path} />
      ))}
      {more > 0 ? (
        <li className={styles.more}>{t('ui.left-out-lines.more', { count: { kind: 'count', value: more } })}</li>
      ) : null}
    </ul>
  );
};

type LineProps = {
  readonly bar: boolean;
  readonly count: LeftOutCount;
  readonly home: null | string;
  readonly open: boolean;
};

// One reason's line. Held back for now with items to show, the sentence is the toggle that unfolds them; otherwise it
// is a plain line.
const Line = ({ bar, count, home, open: initiallyOpen }: LineProps): JSX.Element => {
  const itemsId = useId();
  const [open, setOpen] = useState(initiallyOpen);
  const icon = bar && count.kind === 'for-now' ? <Icon className={styles.icon ?? ''} icon={Info} /> : null;
  const foldable = count.kind === 'for-now' && (count.shown.length > 0 || moreOf(count) > 0);

  return (
    <li className={styles.line} data-foldable={foldable ? true : undefined} data-kind={count.kind}>
      {foldable ? (
        <>
          <button
            aria-controls={itemsId}
            aria-expanded={open}
            className={styles.toggle}
            onClick={() => {
              setOpen(!open);
            }}
            type="button"
          >
            {icon}
            {/* A left-out reason is a bare text key in the core (`crates/core/src/left_out.rs`), never with a `detail`:
                no ProblemText "Details" button is needed inside the toggle. */}
            <span className={styles.sentence}>
              {summaryOf(count)} {tText(count.reason)}
            </span>
            <span className={styles.chevron}>
              <Icon icon={ChevronRight} />
            </span>
          </button>
          <Items count={count} hidden={!open} home={home} id={itemsId} />
        </>
      ) : (
        <span className={styles.summary}>
          {icon}
          <span>{summaryOf(count)} </span>
          <ProblemText text={count.reason} />
        </span>
      )}
    </li>
  );
};

// The counts held back for now first, then the ones kept for good, each part in the core's order.
const ordered = (counts: readonly LeftOutCount[]): readonly LeftOutCount[] => [
  ...counts.filter(({ kind }) => kind === 'for-now'),
  ...counts.filter(({ kind }) => kind === 'kept'),
];

// What a category recognized and did not offer, one line per reason: how many items, how big and why. A line held
// back for now is folded: its sentence is a toggle that shows each item by name, path and size. A count kept for good
// (the version you use) is one quiet line with no item, after the others. Nothing is counted here but the items the
// core did not list: the numbers and the texts come from the core.
export const LeftOutLines = ({
  bar = false,
  counts,
  home = null,
  open = false,
}: LeftOutLinesProps): JSX.Element | null =>
  counts.length === 0 ? null : (
    <ul aria-label={t('ui.left-out-lines.label')} className={styles.lines} data-bar={bar ? true : undefined}>
      {ordered(counts).map((count) => (
        <Line bar={bar} count={count} home={home} key={`${count.kind}:${count.reason.key}`} open={open} />
      ))}
    </ul>
  );
