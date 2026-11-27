import type { JSX } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { Forced } from '../../design/primitives/forced';
import type { LargestRow, OwnRow } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { SizeText } from '../../design/data/size-text/size-text';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { MiddlePath } from '../middle-path/middle-path';
import { honestSize } from '../sizes';
import styles from './own-item-row.module.css';

export type OwnItemRowProps = Forced &
  Partial<VirtualRowProps> & {
    // Whether a row of Files is ticked; a Browse row is never ticked.
    readonly checked?: boolean;
    // "Open" on a folder that opens; without it the row has no button.
    readonly onOpen?: () => void;
    // "Show in Finder"; without it the row has no button.
    readonly onReveal?: () => void;
    // Ticks or unticks an offered row of Files; without it the row has no checkbox.
    readonly onToggle?: () => void;
    // A row of Files (`LargestRow`) or a read-only row of Browse (`OwnRow`).
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
// stands only on a row of Files the core offers; a row of Browse has none, nor a place for one. The size ends the row.
export const OwnItemRow = ({
  checked = false,
  onOpen,
  onReveal,
  onToggle,
  row,
  ...forced
}: OwnItemRowProps): JSX.Element => {
  const { node, offer } = row;
  const size = honestSize(node.sizes);
  const offered = offer.kind === 'offered';
  const kind = text(`core.largest.kind.${row.kind}`);
  const changed =
    node.modified === null ? null : t('ui.own-item-row.changed', { when: { kind: 'date', value: node.modified } });
  // A file's place is cut in the middle when the line is short, so it is a piece of its own.
  const place = isFile(row) ? row.place : null;
  const explained =
    isFile(row) || row.explained.allocated === 0
      ? null
      : t('ui.own-item-row.explained', { size: { kind: 'size', value: row.explained.allocated } });
  const meta = [kind, ...(explained === null ? [] : [explained]), ...(changed === null ? [] : [changed])].join(' · ');
  const note = !offered
    ? text(`core.largest.withheld.${offer.reason}`)
    : isFile(row) && row.unpacked !== null
      ? text('core.largest.unpacked', { folder: row.unpacked })
      : null;

  return (
    <ListRow
      {...forced}
      leading={
        !isFile(row) ? undefined : offered && onToggle !== undefined ? (
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
          {place === null ? (
            <span className={styles.line}>{meta}</span>
          ) : (
            <span className={`${styles.line} ${styles.meta}`}>
              <span>{`${kind} · `}</span>
              <MiddlePath path={place} />
              {changed === null ? null : <span>{` · ${changed}`}</span>}
            </span>
          )}
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
          {onReveal === undefined ? null : (
            <Button
              aria-label={t('ui.folder-row.reveal-label', { name: { kind: 'text', value: node.name } })}
              onClick={onReveal}
              size="small"
              variant="secondary"
            >
              {t('ui.folder-row.reveal')}
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
