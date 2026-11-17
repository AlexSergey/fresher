import { type JSX, useId } from 'react';

import type { Forced } from '../../design/primitives/forced';
import type { NodeInfo, PlaceRow, Sizes } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { type VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import { Icon } from '../../design/icons/icon';
import { ChevronRight } from '../../design/icons/icons';
import { Skeleton } from '../../design/primitives/skeleton/skeleton';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './place-list.module.css';

type RowProps = Forced & Partial<VirtualRowProps>;

// Why a folder was not scanned, in the core's words.
export const notScannedText = ({ notScanned }: NodeInfo): null | string => {
  if (notScanned === null) {
    return null;
  }
  const reason = notScanned.kind === 'unreadable' ? `unreadable.${notScanned.value}` : notScanned.kind;

  return tText({ args: {}, key: `core.storage.not-scanned.${reason}` });
};

// A row opens when it is a folder the scan went into: a file, a link or a folder not read has nothing to show.
export const opens = ({ kind, notScanned }: NodeInfo): boolean => kind === 'folder' && notScanned === null;

const percentOf = (part: number, whole: number): string =>
  whole > 0 ? `${String(Math.min(100, Math.round((part / whole) * 10_000) / 100))}%` : '0%';

// The row's share of the folder as a thin bar, its explained part in Iris; the row's texts say the same in words.
const ShareBar = ({
  explained,
  folder,
  sizes,
}: {
  readonly explained: Sizes;
  readonly folder: Sizes;
  readonly sizes: Sizes;
}): JSX.Element => (
  <span aria-hidden="true" className={styles.share}>
    <span className={styles.part} data-part="row" style={{ width: percentOf(sizes.allocated, folder.allocated) }}>
      {explained.allocated > 0 ? (
        <span
          className={styles.explained}
          data-part="explained"
          style={{ width: percentOf(explained.allocated, sizes.allocated) }}
        />
      ) : null}
    </span>
  </span>
);

// The row of a page still loading: skeleton lines where the name and the size will be.
const LoadingRow = ({ row }: { readonly row: RowProps }): JSX.Element => (
  <div {...row} aria-label={t('ui.place-list.loading')} className={styles.loading} role="listitem">
    <span className={styles.lines}>
      <Skeleton shape="line" />
    </span>
    <span className={styles.value}>
      <Skeleton shape="line" />
    </span>
  </div>
);

export type FolderRowProps = RowProps & {
  // The folder open, whose size the row's bar takes its share of.
  readonly folder: Sizes;
  // `undefined` while its page loads: the row is a skeleton of the same height.
  readonly row: PlaceRow | undefined;
};

// One child of the open folder: its name, why it was not read or how much of it is explained, its share of the folder
// and its size; a chevron when it opens. Never a checkbox: Fresher suggests nothing here.
export const FolderRow = ({ folder, row, ...virtual }: FolderRowProps): JSX.Element => {
  const describedId = useId();

  if (row === undefined) {
    return <LoadingRow row={virtual} />;
  }

  const { explained, node } = row;
  const size = honestSize(node.sizes);
  const why = notScannedText(node);
  const subtitle =
    why ??
    (explained.allocated > 0
      ? t('ui.place-list.explained', { size: { kind: 'size', value: explained.allocated } })
      : undefined);

  return (
    <ListRow
      {...virtual}
      aria-label={t('ui.place-list.row', {
        name: { kind: 'text', value: node.name },
        size: { kind: 'text', value: size.accessibleText },
      })}
      {...(subtitle === undefined ? {} : { 'aria-describedby': describedId })}
      {...(subtitle === undefined ? {} : { subtitle: <span id={describedId}>{subtitle}</span> })}
      title={node.name}
      trailing={
        <>
          <ShareBar explained={explained} folder={folder} sizes={node.sizes} />
          <span className={styles.value}>
            <SizeText {...size} />
          </span>
          <span className={styles.chevron}>{opens(node) ? <Icon icon={ChevronRight} /> : null}</span>
        </>
      }
    />
  );
};
