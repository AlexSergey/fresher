import { type JSX, useId, useState } from 'react';

import type { FootprintItem, FootprintKind } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { PathText } from '../../design/data/path-text/path-text';
import { SizeText } from '../../design/data/size-text/size-text';
import { Icon } from '../../design/icons/icon';
import { ChevronDown, ChevronRight, Info } from '../../design/icons/icons';
import { t, tText } from '../../i18n';
import { levelName } from '../finding-row/texts';
import { honestSize } from '../sizes';
import styles from './footprint-list.module.css';

export type FootprintListProps = {
  // Every file the app would lose, kept ones too, the bundle first.
  readonly items: readonly FootprintItem[];
};

type Section = { readonly items: readonly FootprintItem[]; readonly kind: FootprintKind };

// The items by kind, in the order the kinds first appear.
const sectionsOf = (items: readonly FootprintItem[]): readonly Section[] => {
  const kinds = [...new Set(items.map((item) => item.kind))];

  return kinds.map((kind) => ({ items: items.filter((item) => item.kind === kind), kind }));
};

const Row = ({ item }: { readonly item: FootprintItem }): JSX.Element => {
  const [open, setOpen] = useState(false);
  const pathId = useId();
  const goes =
    item.kept !== null
      ? null
      : item.keepData
        ? t('ui.product.footprint-list.goes.both')
        : t('ui.product.footprint-list.goes.everything');

  return (
    <li className={item.kept === null ? styles.row : `${styles.row} ${styles.kept}`}>
      <div className={styles.head}>
        <DangerBadge label={levelName(item.danger)} level={item.danger} size="small" />
        <span className={styles.goes}>{goes}</span>
        <span className={styles.size}>
          <SizeText {...honestSize(item.sizes)} />
        </span>
      </div>
      {item.kept === null ? null : (
        <p className={styles.reason}>
          <Icon icon={Info} />
          {tText(item.kept.text)}
        </p>
      )}
      <button
        aria-controls={pathId}
        aria-expanded={open}
        className={styles.toggle}
        onClick={() => {
          setOpen(!open);
        }}
        type="button"
      >
        <Icon icon={open ? ChevronDown : ChevronRight} />
        {t('ui.product.footprint-list.path')}
      </button>
      {open ? (
        <div className={styles.path} id={pathId}>
          <PathText path={item.path} />
        </div>
      ) : null}
    </li>
  );
};

// What an app would lose, item by item and grouped by kind: each item's danger and size, with which uninstall choice
// takes it, or why it stays; the path is there on demand. A kept item is muted and is in no uninstall.
export const FootprintList = ({ items }: FootprintListProps): JSX.Element => (
  <div className={styles.list}>
    {sectionsOf(items).map(({ items: own, kind }) => (
      <section aria-label={tText({ args: {}, key: `core.apps.part.${kind}` })} className={styles.section} key={kind}>
        <h3 className={styles.kind}>{tText({ args: {}, key: `core.apps.part.${kind}` })}</h3>
        <ul className={styles.rows}>
          {own.map((item) => (
            <Row item={item} key={item.path} />
          ))}
        </ul>
      </section>
    ))}
  </div>
);
