import { type JSX, useId, useState } from 'react';

import type { MapPart } from '../../ipc/bindings.generated';

import { Icon } from '../../design/icons/icon';
import { ChevronRight } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { PartReasons } from './part-reasons';
import { PartSize } from './part-size';
import { partDescription, partName } from './parts';
import styles from './storage-map.module.css';

export type PartLineProps = {
  readonly part: MapPart;
  // Free's note: how much more macOS can free on its own; `null` when not asked.
  readonly purgeable: null | number;
  // During a scan the size counts toward its value.
  readonly scanning: boolean;
};

// One line of a volume's map: the part's name and size, and on demand what it is; Explained adds what removing
// it frees, the remainder its reasons (plain lines, never a size each). Free carries the purgeable note beside it.
export const PartLine = ({ part, purgeable, scanning }: PartLineProps): JSX.Element => {
  const [open, setOpen] = useState(false);
  const detailsId = useId();
  const nameId = useId();
  const { kind, sizes } = part;

  return (
    <li className={styles.line}>
      <div className={styles.head}>
        <span className={styles.name} id={nameId}>
          {partName(kind)}
        </span>
        <span className={styles.size}>
          <PartSize scanning={scanning} sizes={sizes} />
        </span>
        <button
          aria-controls={detailsId}
          aria-describedby={nameId}
          aria-expanded={open}
          className={styles.toggle}
          onClick={() => {
            setOpen(!open);
          }}
          type="button"
        >
          {t('ui.storage-map.details')}
          <span className={styles.chevron}>
            <Icon icon={ChevronRight} />
          </span>
        </button>
      </div>
      {kind.kind === 'free' && purgeable !== null && purgeable > 0 ? (
        <p className={styles.note}>
          {tText({ args: { size: { kind: 'size', value: purgeable } }, key: 'core.storage.purgeable' })}
        </p>
      ) : null}
      <div className={styles.details} hidden={!open} id={detailsId}>
        <p>{partDescription(kind)}</p>
        {kind.kind === 'explained' ? (
          <p>{t('ui.storage-map.freed', { size: { kind: 'size', value: sizes.reclaimable } })}</p>
        ) : null}
        {kind.kind === 'remainder' && kind.reasons.length > 0 ? <PartReasons reasons={kind.reasons} /> : null}
      </div>
    </li>
  );
};
