import { type JSX, useId, useState } from 'react';

import type { MapPart, RemainderReason } from '../../ipc/bindings.generated';

import { AnimatedNumber } from '../../design/data/animated-number/animated-number';
import { SizeText } from '../../design/data/size-text/size-text';
import { Icon } from '../../design/icons/icon';
import { ChevronRight } from '../../design/icons/icons';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { t, tText } from '../../i18n';
import { formatSize } from '../../i18n/format-size';
import { honestSize } from '../sizes';
import { partDescription, partName, reasonText } from './parts';
import styles from './storage-map.module.css';

export type PartLineProps = {
  readonly part: MapPart;
  // Free's note: how much more macOS can free on its own; `null` when not asked.
  readonly purgeable: null | number;
  // During a scan the size counts toward its value.
  readonly scanning: boolean;
};

const Reason = ({ reason }: { readonly reason: RemainderReason }): JSX.Element => (
  <li>
    {reasonText(reason)}
    {reason.kind === 'unreadable' ? (
      <>
        {' '}
        {t('ui.storage-map.unreadable', { count: { kind: 'count', value: reason.folders } })}
        {reason.needs?.kind === 'all-files' ? <> {t('ui.storage-map.needs.all-files')}</> : null}
      </>
    ) : null}
  </li>
);

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
          {scanning ? (
            <>
              {/* Counts for the eye; assistive technology reads the honest size ("at least 42 GB"). */}
              <span aria-hidden="true">
                <AnimatedNumber format={(value) => formatSize(value, sizes.completeness)} value={sizes.allocated} />
              </span>
              <VisuallyHidden>{honestSize(sizes).accessibleText}</VisuallyHidden>
            </>
          ) : (
            <SizeText {...honestSize(sizes)} />
          )}
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
        {kind.kind === 'remainder' && kind.reasons.length > 0 ? (
          <ul className={styles.reasons}>
            {kind.reasons.map((reason) => (
              <Reason key={reason.kind} reason={reason} />
            ))}
          </ul>
        ) : null}
      </div>
    </li>
  );
};
