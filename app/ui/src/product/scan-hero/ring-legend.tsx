import { type JSX, useId, useState } from 'react';

import type { RingAbout, RingPart } from './ring-parts';

import { Info } from '../../design/icons/icons';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { t, tText } from '../../i18n';
import { PartReasons } from '../storage-map/part-reasons';
import { PartSize } from '../storage-map/part-size';
import { partDescription } from '../storage-map/parts';
import styles from './ring-legend.module.css';

export type RingLegendProps = {
  // The key of the part the pointer or focus is on, in the ring or here.
  readonly highlighted: null | string;
  // The key of the part whose explanation starts open (the gallery's, before anyone presses an "i").
  readonly initiallyOpen?: null | string;
  readonly onHighlight: (key: null | string) => void;
  readonly parts: readonly RingPart[];
  // Free's note: how much more macOS can free on its own; `null` when not asked.
  readonly purgeable: null | number;
  // During a scan the sizes count toward their values.
  readonly scanning: boolean;
};

const textOf = (key: string): string => tText({ args: {}, key });

// What a row's "i" opens: the core's words for a level (what Explained is, then what the level means) or a part (what
// it is; the remainder its reasons, Free the purgeable note).
const About = ({ about, purgeable }: { readonly about: RingAbout; readonly purgeable: null | number }): JSX.Element => {
  if (about.kind === 'level') {
    return (
      <>
        <p>{textOf('core.storage.part.explained.description')}</p>
        <p>{textOf(`core.danger.${about.level}.meaning`)}</p>
      </>
    );
  }

  const { kind } = about.part;

  return (
    <>
      <p>{partDescription(kind)}</p>
      {kind.kind === 'remainder' && kind.reasons.length > 0 ? <PartReasons reasons={kind.reasons} /> : null}
      {kind.kind === 'free' && purgeable !== null && purgeable > 0 ? (
        <p>{tText({ args: { size: { kind: 'size', value: purgeable } }, key: 'core.storage.purgeable' })}</p>
      ) : null}
    </>
  );
};

type RowProps = Omit<RingLegendProps, 'initiallyOpen' | 'parts'> & {
  readonly open: boolean;
  readonly part: RingPart;
};

const Row = ({ highlighted, onHighlight, open: startsOpen, part, purgeable, scanning }: RowProps): JSX.Element => {
  const [open, setOpen] = useState(startsOpen);
  const detailsId = useId();
  const { about, key, label, sizes, tone } = part;
  const enter = (): void => {
    onHighlight(key);
  };
  const leave = (): void => {
    onHighlight(null);
  };

  return (
    <li
      className={styles.row}
      data-highlighted={highlighted === key ? '' : undefined}
      data-key={key}
      onBlur={leave}
      onFocus={enter}
      onPointerEnter={enter}
      onPointerLeave={leave}
    >
      <div className={styles.head}>
        <span aria-hidden="true" className={styles.swatch} data-tone={tone} />
        <span className={styles.label}>{label}</span>
        <span className={styles.size}>
          <PartSize scanning={scanning} sizes={sizes} />
        </span>
        <IconButton
          aria-controls={detailsId}
          aria-expanded={open}
          icon={Info}
          label={t('ui.scan-hero.about', { name: { kind: 'text', value: label } })}
          onClick={() => {
            setOpen(!open);
          }}
          size="small"
        />
      </div>
      <div className={styles.about} hidden={!open} id={detailsId}>
        <About about={about} purgeable={purgeable} />
      </div>
    </li>
  );
};

// The hero's legend: one row per ring part, Free included, in the ring's order: its swatch, name and size, and an "i"
// that opens what it is. A row and its arc highlight each other.
export const RingLegend = ({
  highlighted,
  initiallyOpen = null,
  onHighlight,
  parts,
  purgeable,
  scanning,
}: RingLegendProps): JSX.Element => (
  <ul aria-label={t('ui.scan-hero.legend')} className={styles.legend}>
    {parts.map((part) => (
      <Row
        highlighted={highlighted}
        key={part.key}
        onHighlight={onHighlight}
        open={part.key === initiallyOpen}
        part={part}
        purgeable={purgeable}
        scanning={scanning}
      />
    ))}
  </ul>
);
