import type { JSX } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { CleanupItemView, CleanupSummary, PutBackOutcome } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { Icon } from '../../design/icons/icon';
import { ChevronRight } from '../../design/icons/icons';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { skipCodeText } from '../cleanup-texts';
import { honestSize } from '../sizes';
import styles from './cleanup-row.module.css';

export type CleanupItemRowProps = Partial<VirtualRowProps> & {
  readonly item: CleanupItemView;
  // Puts this item back; offered only while it is in the Trash.
  readonly onPutBack?: () => void;
  // How putting it back just ended, with the core's text; replaces what the history says of it.
  readonly putBack?: null | string;
};

export type CleanupRowProps = {
  readonly onOpen: () => void;
  readonly summary: CleanupSummary;
};

const dateOf = (summary: CleanupSummary): { readonly date: { readonly kind: 'date'; readonly value: number } } => ({
  date: { kind: 'date', value: summary.started },
});

// Where a cleanup's items are now, in a word or two: what was put back first, as it is the user's last act on it.
const stateOf = (summary: CleanupSummary): string => {
  const total = { kind: 'count', value: summary.count } as const;

  if (summary.putBack > 0) {
    const count = { kind: 'count', value: summary.putBack } as const;

    return summary.inTrash === 0
      ? t('ui.cleanup-row.state.put-back', { count, total })
      : t('ui.cleanup-row.state.put-back-in-trash', {
          count,
          left: { kind: 'count', value: summary.inTrash },
          total,
        });
  }
  if (summary.ended === 'interrupted') {
    return t('ui.cleanup-row.state.interrupted');
  }
  if (summary.ended !== 'completed') {
    return t('ui.cleanup-row.state.stopped');
  }
  if (summary.method === 'permanent') {
    return t('ui.cleanup-row.state.deleted');
  }

  return summary.inTrash === 0
    ? t('ui.cleanup-row.state.not-in-trash')
    : t('ui.cleanup-row.state.in-trash', { count: { kind: 'count', value: summary.inTrash }, total });
};

// One cleanup of the history: its date, how it went, how many items, where they are now and its size. Opens it.
export const CleanupRow = ({ onOpen, summary }: CleanupRowProps): JSX.Element => {
  const count = { count: { kind: 'count', value: summary.count } } as const;
  const sizes = summary.method === 'trash' ? summary.moved : summary.deleted;

  return (
    <li className={styles.cleanup}>
      <button className={styles.open} onClick={onOpen} type="button">
        <span className={styles.content}>
          <span className={styles.title}>{t('ui.cleanup-row.title', dateOf(summary))}</span>
          <span className={styles.subtitle}>
            {summary.method === 'trash' ? t('ui.cleanup-row.trash', count) : t('ui.cleanup-row.permanent', count)}
          </span>
        </span>
        <span className={styles.state} data-ended={summary.ended}>
          {stateOf(summary)}
        </span>
        <span className={styles.size}>
          <SizeText {...honestSize(sizes)} />
        </span>
        <Icon icon={ChevronRight} />
      </button>
    </li>
  );
};

const putBackText = (outcome: PutBackOutcome): string =>
  tText({ args: {}, key: `core.cleanup.put-back.${outcome.kind}` });

// What became of an item: where it is now for one moved to the Trash, else its outcome or why it was skipped.
const outcomeOf = (item: CleanupItemView): string => {
  if (item.putBack !== null) {
    return putBackText(item.putBack);
  }
  if (item.trash !== null) {
    return tText({ args: {}, key: `core.cleanup.trash.${item.trash}` });
  }

  return item.outcome.kind === 'skipped'
    ? skipCodeText(item.outcome.reason)
    : tText({ args: {}, key: `core.cleanup.outcome.${item.outcome.kind}` });
};

// The name an item is shown under: the last name of its path, or its category for a tool's item.
const titleOf = (item: CleanupItemView): string =>
  item.path?.split('/').findLast((name) => name !== '') ?? tText({ args: {}, key: `${item.category}.name` });

// One item of a past cleanup: its name, its category and what became of it, its size, and Put back while it is in
// the Trash.
export const CleanupItemRow = ({ item, onPutBack, putBack = null, ...row }: CleanupItemRowProps): JSX.Element => {
  const title = titleOf(item);

  return (
    <ListRow
      {...row}
      // A put back's answer may be long: it takes the whole line.
      subtitle={putBack ?? `${tText({ args: {}, key: `${item.category}.name` })} · ${outcomeOf(item)}`}
      title={title}
      trailing={
        <>
          <span className={styles.size}>
            <SizeText {...honestSize(item.sizes)} />
          </span>
          {item.trash === 'in-trash' && putBack === null && onPutBack !== undefined ? (
            <Button onClick={onPutBack} size="small">
              {t('ui.cleanup-row.put-back')}
            </Button>
          ) : null}
        </>
      }
    />
  );
};
