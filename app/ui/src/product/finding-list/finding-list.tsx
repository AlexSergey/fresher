import { type JSX, useRef } from 'react';

import type { Finding, Refusal } from '../../ipc/bindings.generated';

import { VirtualList } from '../../design/data/virtual-list/virtual-list';
import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { ErrorState } from '../../design/feedback/error-state/error-state';
import { CircleCheck } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { FindingRow, findingRowsClass } from '../finding-row/finding-row';
import styles from './finding-list.module.css';

export type FindingListProps = {
  readonly failure: FindingListFailure | null;
  readonly isSelected: (finding: Finding) => boolean;
  // The list's accessible name.
  readonly label: string;
  // The rows rendered changed: load their pages. "Try again" asks for the last range again.
  readonly onRangeChange: (first: number, last: number) => void;
  // "Show in Finder" on a row the app may not remove; without it the rows offer no button.
  readonly onReveal?: (finding: Finding) => void;
  // A finding the app may not remove is never toggled: its row has no checkbox and Space passes it by.
  readonly onToggle: (finding: Finding) => void;
  // The finding at an index, or `undefined` while its page loads.
  readonly rowAt: (index: number) => Finding | undefined;
  // How many findings the list has; `undefined` until the core answered its first page.
  readonly total: number | undefined;
};

// Why the rows could not be loaded: the core refused with its text, or could not be reached (`Failure` of `ipc/`).
type FindingListFailure = { readonly kind: 'refused'; readonly refusal: Refusal } | { readonly kind: 'unreachable' };

// Skeleton rows shown while the first page loads.
const LOADING_ROWS = 8;

// Pixels: `--size-finding-row`.
const ROW_HEIGHT = 52;

type Range = { readonly first: number; readonly last: number };

const keyOf = (finding: Finding | undefined, index: number): string =>
  finding === undefined ? `loading-${String(index)}` : `finding-${String(finding.id)}`;

const failureText = (failure: FindingListFailure): string =>
  failure.kind === 'refused' ? tText(failure.refusal.text) : t('ui.core.unreachable');

// The column names, for the eye: each row reads its own title, level and size.
const Header = (): JSX.Element => (
  <div aria-hidden="true" className={styles.header}>
    <span className={styles.item}>{t('ui.finding-list.columns.item')}</span>
    <span className={styles.values}>
      <span className={styles.level}>{t('ui.finding-list.columns.level')}</span>
      <span className={styles.size}>{t('ui.finding-list.columns.size')}</span>
    </span>
  </div>
);

// The findings of one list, largest first as the source gives them, in a virtual list of 52 px rows: skeleton rows
// while the first page loads, the empty state when there is nothing, the core's text and "Try again" when a page was
// refused. Rows of a page that arrives fade in; a row the refreshed order moves glides to its new place.
export const FindingList = ({
  failure,
  isSelected,
  label,
  onRangeChange,
  onReveal,
  onToggle,
  rowAt,
  total,
}: FindingListProps): JSX.Element => {
  const rangeRef = useRef<Range>({ first: 0, last: LOADING_ROWS - 1 });
  const loading = total === undefined;

  const body = ((): JSX.Element => {
    if (failure !== null) {
      return (
        <ErrorState
          action={{
            label: t('ui.finding-list.retry'),
            onClick: () => {
              onRangeChange(rangeRef.current.first, rangeRef.current.last);
            },
          }}
          text={failureText(failure)}
          title={t('ui.finding-list.failed.title')}
        />
      );
    }

    if (total === 0) {
      return (
        <EmptyState
          icon={CircleCheck}
          text={t('ui.finding-list.empty.text')}
          title={t('ui.finding-list.empty.title')}
        />
      );
    }

    const itemAt = (index: number): Finding | undefined => (loading ? undefined : rowAt(index));

    return (
      <VirtualList
        count={total ?? LOADING_ROWS}
        getItem={itemAt}
        getKey={(index) => keyOf(itemAt(index), index)}
        label={label}
        onRangeChange={(first, last) => {
          rangeRef.current = { first, last };
          onRangeChange(first, last);
        }}
        onToggle={(index) => {
          const finding = itemAt(index);

          if (finding?.removability === 'removable') {
            onToggle(finding);
          }
        }}
        renderRow={(_, finding, row) => (
          <FindingRow
            {...row}
            finding={finding}
            onReveal={onReveal}
            onToggle={onToggle}
            selected={finding !== undefined && isSelected(finding)}
          />
        )}
        rowHeight={ROW_HEIGHT}
      />
    );
  })();

  return (
    <div aria-busy={loading || undefined} className={`${styles.frame ?? ''} ${findingRowsClass}`}>
      {failure === null && total !== 0 ? <Header /> : null}
      <div className={styles.body}>{body}</div>
    </div>
  );
};
