import type { JSX, KeyboardEvent, MouseEvent } from 'react';

import type { NodeInfo, PlaceRow, Refusal } from '../../ipc/bindings.generated';

import { VirtualList } from '../../design/data/virtual-list/virtual-list';
import { Notice } from '../../design/feedback/notice/notice';
import { t, tText } from '../../i18n';
import { Breadcrumb, type Crumb } from './breadcrumb';
import { FolderRow, opens } from './folder-row';
import styles from './place-list.module.css';

export type PlaceListProps = {
  // The way down, outermost first; the last is the folder open. Empty while the first page loads.
  readonly crumbs: readonly Crumb[];
  readonly failure: null | PlaceFailure;
  // The folder open, as the core describes it; `null` while its first page loads.
  readonly folder: NodeInfo | null;
  // A folder row: opens it, through its single-child folders.
  readonly onOpen: (node: NodeInfo) => void;
  // The rows rendered changed: load their pages.
  readonly onRangeChange: (first: number, last: number) => void;
  // A step of the breadcrumb: opens it as it is.
  readonly onStep: (index: number) => void;
  // The row at an index, largest first, or `undefined` while its page loads.
  readonly rowAt: (index: number) => PlaceRow | undefined;
  // How many rows the folder has; `undefined` until the core answered its first page.
  readonly total: number | undefined;
};

// Why the rows could not be had: the core refused with its text (a scan is running), or could not be reached.
type PlaceFailure = { readonly kind: 'refused'; readonly refusal: Refusal } | { readonly kind: 'unreachable' };

// Skeleton rows shown while the first page loads.
const LOADING_ROWS = 6;

// The most rows in view at once; the list scrolls past them.
const VISIBLE_ROWS = 10;

// Pixels: `--size-row`.
const ROW_HEIGHT = 44;

const failureText = (failure: PlaceFailure): string =>
  failure.kind === 'refused' ? tText(failure.refusal.text) : t('ui.core.unreachable');

const keyOf = (row: PlaceRow | undefined, index: number): string =>
  row === undefined ? `loading-${String(index)}` : `node-${String(row.node.id.state)}-${String(row.node.id.index)}`;

// The row an event came from: its index when the event's target is the row itself or inside it.
const indexOf = (target: EventTarget): null | number => {
  const slot = target instanceof Element ? target.closest<HTMLElement>('[data-index]') : null;

  return slot === null ? null : Number(slot.dataset.index);
};

// Where the space went inside one folder: the way down as a breadcrumb, then its children, largest first, each with
// its share and how much of it is explained. A folder row opens with a click or Enter; nothing here offers a cleanup.
export const PlaceList = ({
  crumbs,
  failure,
  folder,
  onOpen,
  onRangeChange,
  onStep,
  rowAt,
  total,
}: PlaceListProps): JSX.Element => {
  const loading = total === undefined;
  const itemAt = (index: number): PlaceRow | undefined => (loading ? undefined : rowAt(index));
  const count = total ?? LOADING_ROWS;

  const open = (index: null | number): void => {
    const row = index === null ? undefined : itemAt(index);

    if (row !== undefined && opens(row.node)) {
      onOpen(row.node);
    }
  };

  const body = ((): JSX.Element => {
    if (failure !== null) {
      return (
        <Notice title={t('ui.place-list.failed')} tone="warning">
          {failureText(failure)}
        </Notice>
      );
    }

    if (total === 0) {
      return <p className={styles.empty}>{t('ui.place-list.empty')}</p>;
    }

    return (
      // The rows are the list's focusable items, each a list item: a click or Enter on one, heard here, opens it.
      <div
        className={styles.rows}
        onClick={(event: MouseEvent) => {
          open(indexOf(event.target));
        }}
        onKeyDown={(event: KeyboardEvent) => {
          if (
            event.key === 'Enter' &&
            event.target instanceof Element &&
            event.target.getAttribute('role') === 'listitem'
          ) {
            event.preventDefault();
            open(indexOf(event.target));
          }
        }}
        role="presentation"
        style={{ height: Math.min(count, VISIBLE_ROWS) * ROW_HEIGHT }}
      >
        <VirtualList
          count={count}
          getItem={itemAt}
          getKey={(index) => keyOf(itemAt(index), index)}
          label={t('ui.place-list.folder', { folder: { kind: 'text', value: crumbs.at(-1)?.name ?? '' } })}
          onRangeChange={onRangeChange}
          renderRow={(_, row, virtual) => (
            <FolderRow
              {...virtual}
              folder={folder?.sizes ?? { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 }}
              row={row}
            />
          )}
          rowHeight={ROW_HEIGHT}
        />
      </div>
    );
  })();

  return (
    <div aria-busy={(loading && failure === null) || undefined} className={styles.list}>
      {crumbs.length > 0 ? <Breadcrumb crumbs={crumbs} onStep={onStep} /> : null}
      {body}
    </div>
  );
};
