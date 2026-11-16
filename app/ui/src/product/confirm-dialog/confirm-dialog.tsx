import { type JSX, useRef, useState } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { CategoryId, DangerLevel, FindingId, Sizes, Text } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { ListRow } from '../../design/data/list-row/list-row';
import { PathText } from '../../design/data/path-text/path-text';
import { SizeText } from '../../design/data/size-text/size-text';
import { VirtualList } from '../../design/data/virtual-list/virtual-list';
import { Trash2 } from '../../design/icons/icons';
import { Dialog } from '../../design/layout/dialog/dialog';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { levelMeaning, levelName } from '../finding-row/texts';
import { honestSize } from '../sizes';
import styles from './confirm-dialog.module.css';

// What the user confirms, exactly: the UI's own type until plan 008 fills it from the core's cleanup plan (or
// replaces it with a generated type).
export type ConfirmContent = {
  readonly count: number;
  readonly groups: readonly ConfirmGroup[];
  readonly highest: DangerLevel;
  readonly sizes: Sizes;
};

export type ConfirmDialogProps = {
  readonly content: ConfirmContent;
  // Escape, the close button and Cancel: the owner closes it.
  readonly onCancel: () => void;
  readonly onConfirm: (method: RemovalMethod) => void;
  readonly open: boolean;
};

type ConfirmGroup = {
  readonly category: CategoryId;
  readonly count: number;
  readonly danger: DangerLevel;
  readonly items: readonly ConfirmItem[];
  readonly sizes: Sizes;
};

type ConfirmItem = {
  readonly danger: DangerLevel;
  readonly id: FindingId;
  readonly sizes: Sizes;
  // The path, or an external tool's object as text.
  readonly subject: string;
  readonly title: Text;
};

type RemovalMethod = 'permanent' | 'trash';

// A group lists up to this many items as they are; more go into a virtual list.
const PLAIN_ITEMS = 50;

// Pixels: `--size-row`.
const ITEM_HEIGHT = 44;

const ItemRow = ({ item, ...row }: Partial<VirtualRowProps> & { readonly item: ConfirmItem }): JSX.Element => (
  <ListRow
    {...row}
    subtitle={<PathText path={item.subject} />}
    title={tText(item.title)}
    trailing={
      <>
        <DangerBadge label={levelName(item.danger)} level={item.danger} size="small" />
        <span className={styles.size}>
          <SizeText {...honestSize(item.sizes)} />
        </span>
      </>
    }
  />
);

const Items = ({ group, label }: { readonly group: ConfirmGroup; readonly label: string }): JSX.Element =>
  group.items.length > PLAIN_ITEMS ? (
    <div className={styles.long}>
      <VirtualList
        count={group.items.length}
        getItem={(index) => group.items[index]}
        getKey={(index) => String(group.items[index]?.id ?? index)}
        label={label}
        renderRow={(_, item, row) =>
          item === undefined ? <ListRow {...row} title="" /> : <ItemRow {...row} item={item} />
        }
        rowHeight={ITEM_HEIGHT}
      />
    </div>
  ) : (
    <div aria-label={label} role="list">
      {group.items.map((item) => (
        <ItemRow item={item} key={item.id} />
      ))}
    </div>
  );

const Group = ({ group }: { readonly group: ConfirmGroup }): JSX.Element => {
  const name = tText({ args: {}, key: `${group.category}.name` });

  return (
    <section aria-label={name} className={styles.group}>
      <header className={styles.groupHeader}>
        <h3 className={styles.groupName}>{name}</h3>
        <DangerBadge label={levelName(group.danger)} level={group.danger} size="small" />
        <span className={styles.groupCount}>
          {t('ui.confirm.group-count', { count: { kind: 'count', value: group.count } })}
        </span>
        <span className={styles.size}>
          <SizeText {...honestSize(group.sizes)} />
        </span>
      </header>
      <p className={styles.lost}>{tText({ args: {}, key: `${group.category}.if-removed` })}</p>
      <Items group={group} label={t('ui.confirm.items', { category: { kind: 'text', value: name } })} />
    </section>
  );
};

// The confirmation of a removal: the action and the count, the highest danger level with its sentence, the total
// size, the exact items grouped by category with what each group loses, and the permanent choice, off. Cancel has the
// first focus; no key confirms. Static by the safety rule: only the dialog's own entrance moves.
export const ConfirmDialog = ({ content, onCancel, onConfirm, open }: ConfirmDialogProps): JSX.Element | null => {
  const cancelRef = useRef<HTMLButtonElement>(null);
  const [permanent, setPermanent] = useState(false);
  const [wasOpen, setWasOpen] = useState(open);

  // Every opening starts from the Trash.
  if (open !== wasOpen) {
    setWasOpen(open);

    if (open) {
      setPermanent(false);
    }
  }

  const count = { kind: 'count', value: content.count } as const;
  const level = levelName(content.highest);
  const size = honestSize(content.sizes);

  return (
    <Dialog
      closeLabel={t('ui.confirm.close')}
      description={t('ui.confirm.description', {
        count,
        level: { kind: 'text', value: level },
        size: { kind: 'text', value: size.accessibleText },
      })}
      footer={
        <>
          <Button onClick={onCancel} ref={cancelRef} variant="secondary">
            {t('ui.confirm.cancel')}
          </Button>
          <Button
            icon={Trash2}
            onClick={() => {
              onConfirm(permanent ? 'permanent' : 'trash');
            }}
            variant={permanent ? 'danger' : 'primary'}
          >
            {permanent ? t('ui.confirm.delete') : t('ui.confirm.trash')}
          </Button>
        </>
      }
      initialFocus={cancelRef}
      onClose={onCancel}
      open={open}
      size="large"
      title={permanent ? t('ui.confirm.title-permanent', { count }) : t('ui.confirm.title-trash', { count })}
    >
      <div className={styles.content}>
        <div className={styles.summary}>
          <DangerBadge description={levelMeaning(content.highest)} label={level} level={content.highest} size="large" />
          <div className={styles.total}>
            <span className={styles.totalLabel}>{t('ui.confirm.total')}</span>
            <SizeText {...size} size="display" />
          </div>
        </div>
        {content.groups.map((group) => (
          <Group group={group} key={group.category} />
        ))}
        <div className={styles.choice}>
          <Checkbox checked={permanent} label={t('ui.confirm.permanent')} onChange={setPermanent} />
        </div>
      </div>
    </Dialog>
  );
};
