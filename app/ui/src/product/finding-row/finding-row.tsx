import { type JSX, useId } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { Forced } from '../../design/primitives/forced';
import type { Finding, Subject } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { ListRow } from '../../design/data/list-row/list-row';
import { PathText } from '../../design/data/path-text/path-text';
import { SizeText } from '../../design/data/size-text/size-text';
import { Settings } from '../../design/icons/icons';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { Menu, type MenuItem } from '../../design/primitives/menu/menu';
import { Skeleton } from '../../design/primitives/skeleton/skeleton';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './finding-row.module.css';
import { RemovabilityNote } from './removability-note';
import { levelName, removabilityName } from './texts';

export type FindingRowProps = Forced &
  Partial<VirtualRowProps> & {
    // `undefined` while its page loads: the row is a skeleton of the same height.
    readonly finding: Finding | undefined;
    // The items of the row's settings menu; without any the row has no menu.
    readonly menu?: readonly MenuItem[] | undefined;
    // The menu starts open, without taking the focus: for the gallery.
    readonly menuOpen?: boolean | undefined;
    // "Show in Finder" on a row the app may not remove; without it the row offers no button.
    readonly onReveal?: ((finding: Finding) => void) | undefined;
    readonly onToggle: (finding: Finding) => void;
    readonly selected: boolean;
    // A skeleton row keeps the menu column's room when its list has menus.
    readonly withMenu?: boolean;
  };

// The class of a frame of finding rows: their height (52 px) and the widths of the level and size columns.
export const findingRowsClass = styles.rows ?? '';

const SubjectText = ({ subject }: { readonly subject: Subject }): JSX.Element =>
  subject.kind === 'path' ? (
    <PathText path={subject.value} />
  ) : (
    <span className={styles.external}>
      {t('ui.finding-list.external', {
        id: { kind: 'text', value: subject.value.id },
        tool: { kind: 'text', value: subject.value.tool },
      })}
    </span>
  );

// The row of a page still loading: skeleton lines where the title, the subject and the values will be.
const LoadingRow = ({
  row,
  withMenu,
}: {
  readonly row: Forced & Partial<VirtualRowProps>;
  readonly withMenu: boolean;
}): JSX.Element => (
  <div {...row} aria-label={t('ui.finding-list.loading')} className={styles.loading} role="listitem">
    <span className={styles.lines}>
      <span className={styles.title}>
        <Skeleton shape="line" />
      </span>
      <span className={styles.subject}>
        <Skeleton shape="line" />
      </span>
    </span>
    <span className={styles.level}>
      <span className={styles.badge}>
        <Skeleton shape="line" />
      </span>
    </span>
    <span className={styles.value}>
      <span className={styles.amount}>
        <Skeleton shape="line" />
      </span>
    </span>
    {withMenu ? <span aria-hidden="true" className={styles.menu} /> : null}
  </div>
);

// One finding in a row (52 px inside a frame of `findingRowsClass`): its checkbox, its title over its
// path, its danger level and its size. Its accessible name reads title, level and size; its description is the
// reason, which the level's badge also shows in a tooltip. A selected High row lights its edge in High's colour.
// A finding the core says the app may not remove keeps its level and size but has no checkbox and is never shown
// selected; its removability's word stands before the level, with what the user can do on demand. With `menu`, a
// settings button ends the row; it is a tab stop only in the list's focused row.
export const FindingRow = ({
  finding,
  menu,
  menuOpen,
  onReveal,
  onToggle,
  selected,
  withMenu = false,
  ...row
}: FindingRowProps): JSX.Element => {
  const reasonId = useId();

  if (finding === undefined) {
    return <LoadingRow row={row} withMenu={withMenu} />;
  }

  const title = tText(finding.title);
  const reason = tText(finding.reason);
  const level = levelName(finding.danger);
  const size = honestSize(finding.sizes);
  const { removability } = finding;
  const named = {
    level: { kind: 'text', value: level },
    size: { kind: 'text', value: size.accessibleText },
    title: { kind: 'text', value: title },
  } as const;

  return (
    <ListRow
      {...row}
      aria-describedby={reasonId}
      aria-label={
        removability === 'removable'
          ? t('ui.finding-list.row', named)
          : t('ui.finding-list.row-kept', {
              ...named,
              removability: { kind: 'text', value: removabilityName(removability) },
            })
      }
      {...(finding.danger === 'high' ? { edgeLevel: 'high' as const } : {})}
      leading={
        removability === 'removable' ? (
          <Checkbox
            checked={selected}
            hideLabel
            label={t('ui.finding-list.select', { title: { kind: 'text', value: title } })}
            onChange={() => {
              onToggle(finding);
            }}
            tabIndex={-1}
          />
        ) : (
          // The checkbox's place stays empty, so the titles of a list line up.
          <span aria-hidden="true" className={styles.noCheckbox} />
        )
      }
      selected={removability === 'removable' && selected}
      subtitle={
        <>
          <SubjectText subject={finding.subject} />
          <span hidden id={reasonId}>
            {reason}
          </span>
        </>
      }
      title={title}
      trailing={
        <>
          {removability === 'removable' ? null : (
            <RemovabilityNote finding={finding} onReveal={onReveal} removability={removability} />
          )}
          <span className={styles.level}>
            <Tooltip content={reason} repeatsName>
              {() => <DangerBadge label={level} level={finding.danger} size="small" />}
            </Tooltip>
          </span>
          <span className={styles.value}>
            <SizeText {...size} />
          </span>
          {menu === undefined ? null : (
            <span aria-hidden={menu.length === 0 ? true : undefined} className={styles.menu}>
              {menu.length === 0 ? null : (
                <Menu
                  defaultOpen={menuOpen}
                  icon={Settings}
                  items={menu}
                  label={t('ui.finding-list.options', { title: { kind: 'text', value: title } })}
                  // The menu lives only in the list's roving model: the list gives the row its tabIndex.
                  tabIndex={row.tabIndex ?? -1}
                />
              )}
            </span>
          )}
        </>
      }
    />
  );
};
