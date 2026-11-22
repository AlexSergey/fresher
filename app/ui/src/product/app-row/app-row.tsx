import { type JSX, type KeyboardEvent, useId, useState } from 'react';

import type { AppSummary } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Icon } from '../../design/icons/icon';
import { ChevronDown, ChevronRight, Info } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './app-row.module.css';

export type AppRowProps = {
  // Opens the app's page.
  readonly onOpen: () => void;
  // The "What it takes" breakdown starts open.
  readonly partsOpen?: boolean;
  readonly summary: AppSummary;
};

// One app of the Apps list: its name and version, the core's sentence, why it is not offered when it is not, and on
// demand what it takes by kind. A click or Enter on the row opens the app's page; the size ends the row.
export const AppRow = ({ onOpen, partsOpen = false, summary }: AppRowProps): JSX.Element => {
  const [open, setOpen] = useState(partsOpen);
  const partsId = useId();
  const { name, offer, parts, sizes, version } = summary;

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>): void => {
    if (event.key === 'Enter' && event.target === event.currentTarget) {
      onOpen();
    }
  };

  return (
    // eslint-disable-next-line jsx-a11y-x/no-noninteractive-element-interactions, jsx-a11y-x/no-noninteractive-tabindex -- the row is a list item that opens the app's page; the disclosure inside is its own button
    <div className={styles.row} onClick={onOpen} onKeyDown={onKeyDown} role="listitem" tabIndex={0}>
      <div className={styles.head}>
        <div className={styles.text}>
          <span className={styles.name}>
            {name}
            {version === null ? null : (
              <span className={styles.version}>
                {t('ui.product.app-row.version', { version: { kind: 'text', value: version } })}
              </span>
            )}
          </span>
          <span className={styles.line}>{tText(summary.summary)}</span>
          {offer.kind === 'not-offered' ? (
            <span className={`${styles.line} ${styles.muted}`}>
              <Icon icon={Info} />
              {tText(offer.text)}
            </span>
          ) : null}
        </div>
        <span className={styles.size}>
          <SizeText {...honestSize(sizes)} />
        </span>
      </div>
      <button
        aria-controls={partsId}
        aria-expanded={open}
        className={styles.toggle}
        onClick={(event) => {
          event.stopPropagation();
          setOpen(!open);
        }}
        type="button"
      >
        <Icon icon={open ? ChevronDown : ChevronRight} />
        {t('ui.product.app-row.parts')}
      </button>
      {open ? (
        <ul className={styles.parts} id={partsId}>
          {parts.map(({ kind, sizes: partSizes }) => (
            <li className={styles.part} key={kind}>
              <span>{tText({ args: {}, key: `core.apps.part.${kind}` })}</span>
              <SizeText {...honestSize(partSizes)} />
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
};
