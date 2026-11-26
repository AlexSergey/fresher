import { type JSX, type Ref, useEffect, useId, useRef, useState } from 'react';

import type { VirtualRowProps } from '../../design/data/virtual-list/virtual-list';
import type { UiKey } from '../../i18n/types';
import type {
  ActionKind,
  CleanupPlanView,
  LeftOut,
  Method,
  PlanGroup,
  PlanItemView,
} from '../../ipc/bindings.generated';

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
import { skipReasonText } from '../cleanup-texts';
import { levelMeaning, levelName } from '../finding-row/texts';
import { honestSize } from '../sizes';
import styles from './confirm-dialog.module.css';

export type ConfirmDialogProps = {
  // Escape, the close button and Cancel: the owner closes it.
  readonly onCancel: () => void;
  // The method chosen, and whether the High tick was ticked (always false for a plan below High).
  readonly onConfirm: (method: Method, highRiskUnderstood: boolean) => void;
  readonly open: boolean;
  // The core's plan, shown exactly: every figure is the plan's own.
  readonly plan: CleanupPlanView;
};

// A group lists up to this many items as they are; more go into a virtual list.
const PLAIN_ITEMS = 50;

// Pixels: `--size-row`.
const ITEM_HEIGHT = 44;

// How a group's items go, by the method chosen: a tool's items go the same way either way.
const HOW: Readonly<Record<ActionKind, Readonly<Record<Method, UiKey>>>> = {
  empty: { permanent: 'ui.confirm.how.empty-permanent', trash: 'ui.confirm.how.empty-trash' },
  remove: { permanent: 'ui.confirm.how.remove-permanent', trash: 'ui.confirm.how.remove-trash' },
  tool: { permanent: 'ui.confirm.how.tool', trash: 'ui.confirm.how.tool' },
};

type MoreControl = {
  // The id of the list the button opens.
  readonly controls: string;
  readonly expanded: boolean;
  readonly onToggle: () => void;
};

// The button under an item's path that says how many more files go with it; the list opens on request.
const MoreButton = ({ control, item }: { readonly control: MoreControl; readonly item: PlanItemView }): JSX.Element => {
  const files = t('ui.confirm.more-files', { count: { kind: 'count', value: item.more.length } });

  // The name says whose files they are, so the buttons of several together items differ.
  return (
    <button
      aria-controls={control.controls}
      aria-expanded={control.expanded}
      aria-label={t('ui.confirm.more-files-of', {
        files: { kind: 'text', value: files },
        item: { kind: 'text', value: tText(item.title) },
      })}
      className={styles.moreToggle}
      onClick={control.onToggle}
      type="button"
    >
      {files}
    </button>
  );
};

// Every other path of an item that goes together, one line each.
const MorePaths = ({
  id,
  item,
  ref,
}: {
  readonly id: string;
  readonly item: PlanItemView;
  readonly ref?: Ref<HTMLUListElement>;
}): JSX.Element => (
  <ul
    aria-label={t('ui.confirm.more-list', { item: { kind: 'text', value: tText(item.title) } })}
    className={styles.morePaths}
    id={id}
    ref={ref}
  >
    {item.more.map((path) => (
      <li key={path}>
        <PathText path={path} />
      </li>
    ))}
  </ul>
);

const ItemRow = ({
  item,
  more,
  ...row
}: Partial<VirtualRowProps> & {
  readonly item: PlanItemView;
  readonly more?: MoreControl | undefined;
}): JSX.Element => (
  <ListRow
    {...row}
    subtitle={
      more === undefined ? (
        item.path === null ? undefined : (
          <PathText path={item.path} />
        )
      ) : (
        <span className={styles.subtitle}>
          {item.path === null ? null : <PathText path={item.path} />}
          <MoreButton control={more} item={item} />
        </span>
      )
    }
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

// A plain group's item: its row, and under it the other paths when asked for.
const PlainItem = ({ item }: { readonly item: PlanItemView }): JSX.Element => {
  const id = useId();
  const [expanded, setExpanded] = useState(false);

  if (item.more.length === 0) {
    return <ItemRow item={item} />;
  }

  return (
    <>
      <ItemRow
        item={item}
        more={{
          controls: id,
          expanded,
          onToggle: (): void => {
            setExpanded(!expanded);
          },
        }}
      />
      {expanded ? (
        <div role="listitem">
          <MorePaths id={id} item={item} />
        </div>
      ) : null}
    </>
  );
};

// A virtual row is one fixed height, so the other paths of an item open below the list, not in its row.
const VirtualItems = ({ group, label }: { readonly group: PlanGroup; readonly label: string }): JSX.Element => {
  const id = useId();
  const listRef = useRef<HTMLUListElement>(null);
  const [openedId, setOpenedId] = useState<null | number>(null);
  const opened = group.items.find((item) => item.finding === openedId) ?? null;

  // The list opens below the virtual box, maybe out of view: bring it in.
  useEffect(() => {
    listRef.current?.scrollIntoView({ block: 'nearest' });
  }, [openedId]);

  return (
    <>
      <div className={styles.long}>
        <VirtualList
          count={group.items.length}
          getItem={(index) => group.items[index]}
          getKey={(index) => String(group.items[index]?.finding ?? index)}
          label={label}
          renderRow={(_, item, row) =>
            item === undefined ? (
              <ListRow {...row} title="" />
            ) : (
              <ItemRow
                {...row}
                item={item}
                more={
                  item.more.length === 0
                    ? undefined
                    : {
                        controls: id,
                        expanded: openedId === item.finding,
                        onToggle: (): void => {
                          setOpenedId(openedId === item.finding ? null : item.finding);
                        },
                      }
                }
              />
            )
          }
          rowHeight={ITEM_HEIGHT}
        />
      </div>
      {opened === null ? null : <MorePaths id={id} item={opened} ref={listRef} />}
    </>
  );
};

const Items = ({ group, label }: { readonly group: PlanGroup; readonly label: string }): JSX.Element =>
  group.items.length > PLAIN_ITEMS ? (
    <VirtualItems group={group} label={label} />
  ) : (
    <div aria-label={label} role="list">
      {group.items.map((item) => (
        <PlainItem item={item} key={item.finding} />
      ))}
    </div>
  );

const Group = ({ group, method }: { readonly group: PlanGroup; readonly method: Method }): JSX.Element => {
  const name = tText({ args: {}, key: `${group.category}.name` });
  const { summary } = group;

  return (
    <section aria-label={name} className={styles.group}>
      <header className={styles.groupHeader}>
        <h3 className={styles.groupName}>{name}</h3>
        <DangerBadge label={levelName(summary.highest)} level={summary.highest} size="small" />
        <span className={styles.groupCount}>
          {t('ui.confirm.group-count', { count: { kind: 'count', value: summary.count } })}
        </span>
        <span className={styles.size}>
          <SizeText {...honestSize(summary.sizes)} />
        </span>
      </header>
      <p className={styles.how} data-irreversible={group.action === 'tool' || method === 'permanent' || undefined}>
        {t(HOW[group.action][method])}
      </p>
      <p className={styles.lost}>{tText({ args: {}, key: `${group.category}.if-removed` })}</p>
      <Items group={group} label={t('ui.confirm.items', { category: { kind: 'text', value: name } })} />
    </section>
  );
};

// What the selection held that the plan leaves out, each with the core's reason: not in the count.
const LeftOutList = ({ items }: { readonly items: readonly LeftOut[] }): JSX.Element => (
  <section aria-label={t('ui.confirm.left-out')} className={styles.group}>
    <h3 className={styles.groupName}>{t('ui.confirm.left-out')}</h3>
    <ul className={styles.leftOut}>
      {items.map((item) => (
        <li className={styles.leftOutItem} key={item.finding}>
          <span className={styles.leftOutTitle}>{tText(item.title)}</span>
          <span className={styles.leftOutReason}>{skipReasonText(item.reason)}</span>
        </li>
      ))}
    </ul>
  </section>
);

// How the plan's items go: each by its tool (nothing can be put back), each to the Trash with no other choice (an
// uninstall), or to the Trash unless the user chooses a permanent delete. The core's plan says which.
type Way = 'choice' | 'tools' | 'trash';

const wayOf = (plan: CleanupPlanView): Way => {
  if (plan.permanentAvailable) {
    return 'choice';
  }

  return plan.groups.every((group) => group.action === 'tool') ? 'tools' : 'trash';
};

const titleOf = (plan: CleanupPlanView, way: Way, permanent: boolean): string => {
  const count = { count: { kind: 'count', value: plan.summary.count } } as const;

  if (way === 'tools') {
    return t('ui.confirm.title-tool', count);
  }

  return permanent ? t('ui.confirm.title-permanent', count) : t('ui.confirm.title-trash', count);
};

// The confirmation of a removal: the action and the count, the highest danger level with its sentence, the total size,
// the exact items grouped by category and by how they go, with what each group loses, what will not be removed and why,
// the High tick of a High plan, off, and the permanent choice, off: these two sit with the buttons, below the items
// that scroll, so they are always in sight. Every figure is the core's plan: nothing is summed
// here. A plan whose items were all left out has no confirm button. Cancel has the first focus; no key confirms; a High
// plan cannot be confirmed until its tick is ticked. Static by the safety rule: only the dialog's own entrance moves.
export const ConfirmDialog = ({ onCancel, onConfirm, open, plan }: ConfirmDialogProps): JSX.Element | null => {
  const cancelRef = useRef<HTMLButtonElement>(null);
  const highId = useId();
  const [permanent, setPermanent] = useState(false);
  const [understood, setUnderstood] = useState(false);
  const [wasOpen, setWasOpen] = useState(open);

  // Every opening starts from the Trash, with the High tick off.
  if (open !== wasOpen) {
    setWasOpen(open);

    if (open) {
      setPermanent(false);
      setUnderstood(false);
    }
  }

  const { summary } = plan;
  const way = wayOf(plan);
  const method: Method = permanent && way === 'choice' ? 'permanent' : 'trash';
  const level = levelName(summary.highest);
  const size = honestSize(summary.sizes);
  // A plan of tool items only goes by their tools: it cannot be put back, whatever the method.
  const irreversible = method === 'permanent' || way === 'tools';
  // A High plan asks for its own tick, whatever its way; the core refuses it without.
  const high = summary.highest === 'high';
  const blocked = high && !understood;

  // Every item was left out: there is nothing to confirm, only the reasons to read.
  if (summary.count === 0) {
    return (
      <Dialog
        closeLabel={t('ui.confirm.close')}
        description={t('ui.confirm.nothing-text')}
        footer={
          <Button onClick={onCancel} ref={cancelRef} variant="secondary">
            {t('ui.confirm.back')}
          </Button>
        }
        initialFocus={cancelRef}
        onClose={onCancel}
        open={open}
        size="large"
        title={t('ui.confirm.nothing')}
      >
        <div className={styles.content}>
          <LeftOutList items={plan.leftOut} />
        </div>
      </Dialog>
    );
  }

  return (
    <Dialog
      closeLabel={t('ui.confirm.close')}
      description={t('ui.confirm.description', {
        count: { kind: 'count', value: summary.count },
        level: { kind: 'text', value: level },
        size: { kind: 'text', value: size.accessibleText },
      })}
      footer={
        <div className={styles.actions}>
          {high ? (
            <div className={styles.high} id={highId}>
              <Checkbox
                checked={understood}
                description={t('ui.confirm.high-description')}
                label={t('ui.confirm.high')}
                onChange={setUnderstood}
              />
            </div>
          ) : null}
          {way === 'tools' ? null : (
            <div className={styles.choice}>
              {method === 'trash' ? <p className={styles.space}>{t('ui.confirm.space-later')}</p> : null}
              {way === 'choice' ? (
                <Checkbox
                  checked={permanent}
                  description={t('ui.confirm.permanent-description')}
                  label={t('ui.confirm.permanent')}
                  onChange={setPermanent}
                />
              ) : null}
            </div>
          )}
          <div className={styles.buttons}>
            <Button onClick={onCancel} ref={cancelRef} variant="secondary">
              {t('ui.confirm.cancel')}
            </Button>
            <Button
              aria-describedby={blocked ? highId : undefined}
              disabled={blocked}
              icon={Trash2}
              onClick={() => {
                onConfirm(method, high && understood);
              }}
              variant={irreversible ? 'danger' : 'primary'}
            >
              {way === 'tools'
                ? t('ui.confirm.remove')
                : t(method === 'permanent' ? 'ui.confirm.delete' : 'ui.confirm.trash')}
            </Button>
          </div>
        </div>
      }
      initialFocus={cancelRef}
      onClose={onCancel}
      open={open}
      size="large"
      title={titleOf(plan, way, method === 'permanent')}
    >
      <div className={styles.content}>
        <div className={styles.summary}>
          <DangerBadge description={levelMeaning(summary.highest)} label={level} level={summary.highest} size="large" />
          <div className={styles.total}>
            <span className={styles.totalLabel}>{t('ui.confirm.total')}</span>
            <SizeText {...size} size="display" />
          </div>
        </div>
        {plan.groups.map((group) => (
          <Group group={group} key={`${group.category} ${group.action}`} method={method} />
        ))}
        {plan.leftOut.length === 0 ? null : <LeftOutList items={plan.leftOut} />}
      </div>
    </Dialog>
  );
};
