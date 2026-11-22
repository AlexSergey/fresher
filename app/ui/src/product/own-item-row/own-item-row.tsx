import type { JSX } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { Forced } from '../../design/primitives/forced';
import type { LargestRow, OwnRow } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './own-item-row.module.css';

export type OwnItemRowProps = Forced &
  Partial<VirtualRowProps> & {
    readonly checked: boolean;
    // "Open" on a folder that opens; without it the row has no button.
    readonly onOpen?: () => void;
    // Ticks or unticks an offered row; without it the row has no checkbox.
    readonly onToggle?: () => void;
    // A row of the Files list or of a folder.
    readonly row: LargestRow | OwnRow;
  };

// The class of a frame of these rows: their height, for the lines under the name.
export const ownItemRowsClass = styles.rows ?? '';

const isFile = (row: LargestRow | OwnRow): row is LargestRow => 'place' in row;

const text = (key: string, args: Readonly<Record<string, string>> = {}): string =>
  tText({
    args: Object.fromEntries(Object.entries(args).map(([name, value]) => [name, { kind: 'text', value }])),
    key,
  });

// One item of the Largest screen: its name over its kind, where it is (or what of it Scan explains) and when it
// changed, then the folder an archive may have been unpacked into, or why the item cannot be chosen. The checkbox
// stands only on a row the core offers; the size ends the row.
export const OwnItemRow = ({ checked, onOpen, onToggle, row, ...forced }: OwnItemRowProps): JSX.Element => {
  const { node, offer } = row;
  const size = honestSize(node.sizes);
  const offered = offer.kind === 'offered';
  const where = isFile(row)
    ? row.place
    : row.explained.allocated === 0
      ? null
      : t('ui.own-item-row.explained', { size: { kind: 'size', value: row.explained.allocated } });
  const meta = [
    text(`core.largest.kind.${row.kind}`),
    ...(where === null ? [] : [where]),
    ...(node.modified === null ? [] : [t('ui.own-item-row.changed', { when: { kind: 'date', value: node.modified } })]),
  ].join(' · ');
  const note = !offered
    ? text(`core.largest.withheld.${offer.reason}`)
    : isFile(row) && row.unpacked !== null
      ? text('core.largest.unpacked', { folder: row.unpacked })
      : null;

  return (
    <ListRow
      {...forced}
      leading={
        offered && onToggle !== undefined ? (
          <Checkbox
            checked={checked}
            hideLabel
            label={t('ui.own-item-row.select', {
              name: { kind: 'text', value: node.name },
              size: { kind: 'text', value: size.accessibleText },
            })}
            onChange={onToggle}
            tabIndex={-1}
          />
        ) : (
          // The checkbox's place stays empty, so the names of a list line up.
          <span aria-hidden="true" className={styles.noCheckbox} />
        )
      }
      selected={offered && checked}
      subtitle={
        <>
          <span className={styles.line}>{meta}</span>
          {note === null ? null : <span className={styles.line}>{note}</span>}
        </>
      }
      title={node.name}
      trailing={
        <>
          {onOpen === undefined ? null : (
            <Button onClick={onOpen} size="small">
              {t('ui.own-item-row.open')}
            </Button>
          )}
          <span className={styles.value}>
            <SizeText {...size} />
          </span>
        </>
      }
    />
  );
};
