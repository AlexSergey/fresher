import { type JSX, useId, useState } from 'react';

import type { SummaryKind } from '../../ipc/bindings.generated';
import type { DiskPart } from './disk-parts';

import { t, tText } from '../../i18n';
import { PartSize } from '../storage-map/part-size';
import { AboutButton } from './about-button';
import styles from './disk-legend.module.css';

export type DiskLegendProps = {
  // The kind of the part the pointer or focus is on, in the ring or here.
  readonly highlighted: null | SummaryKind;
  // The kind of the part whose explanation starts open (the gallery's).
  readonly initiallyOpen?: null | SummaryKind;
  readonly onHighlight: (kind: null | SummaryKind) => void;
  readonly parts: readonly DiskPart[];
  // Free's note: how much more macOS can free on its own; `null` when not asked.
  readonly purgeable: null | number;
  // Before the first scan what can be freed is not known yet.
  readonly scanned: boolean;
};

type RowProps = Omit<DiskLegendProps, 'initiallyOpen' | 'parts'> & {
  readonly open: boolean;
  readonly part: DiskPart;
};

const Row = ({ highlighted, onHighlight, open: startsOpen, part, purgeable, scanned }: RowProps): JSX.Element => {
  const [open, setOpen] = useState(startsOpen);
  const detailsId = useId();
  const unknown = part.kind === 'can-be-freed' && !scanned;
  const { description, kind, name, sizes } = part;
  const enter = (): void => {
    onHighlight(kind);
  };
  const leave = (): void => {
    onHighlight(null);
  };

  return (
    <li
      className={styles.row}
      data-highlighted={highlighted === kind ? '' : undefined}
      data-key={kind}
      onBlur={leave}
      onFocus={enter}
      onPointerEnter={enter}
      onPointerLeave={leave}
    >
      <div className={styles.head}>
        <span aria-hidden="true" className={styles.swatch} data-tone={part.tone} />
        <span className={styles.name}>{name}</span>
        <span className={styles.size} data-unknown={unknown ? '' : undefined}>
          {unknown ? t('ui.disk-card.not-scanned') : <PartSize scanning={false} sizes={sizes} />}
        </span>
        <AboutButton
          controls={detailsId}
          expanded={open}
          name={name}
          onToggle={() => {
            setOpen(!open);
          }}
        />
      </div>
      <div className={styles.about} hidden={!open} id={detailsId}>
        <p>{description}</p>
        {kind === 'free' && purgeable !== null && purgeable > 0 ? (
          <p>{tText({ args: { size: { kind: 'size', value: purgeable } }, key: 'core.storage.purgeable' })}</p>
        ) : null}
      </div>
    </li>
  );
};

// The disk card's legend: a row per part in two columns, each with its swatch, name, size and an "i" that opens what
// the part is. A row and its arc highlight each other.
export const DiskLegend = ({
  highlighted,
  initiallyOpen = null,
  onHighlight,
  parts,
  purgeable,
  scanned,
}: DiskLegendProps): JSX.Element => (
  <ul aria-label={t('ui.disk-card.legend')} className={styles.legend}>
    {parts.map((part) => (
      <Row
        highlighted={highlighted}
        key={part.kind}
        onHighlight={onHighlight}
        open={part.kind === initiallyOpen}
        part={part}
        purgeable={purgeable}
        scanned={scanned}
      />
    ))}
  </ul>
);
