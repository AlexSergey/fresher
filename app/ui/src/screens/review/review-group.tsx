import type { CSSProperties, JSX } from 'react';

import type { Finding, FindingId } from '../../ipc/bindings.generated';
import type { Picked } from '../../state/selection';
import type { ReviewGroup } from './review-groups';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { Surface } from '../../design/layout/surface/surface';
import { t, tText } from '../../i18n';
import { FindingList } from '../../product/finding-list/finding-list';
import { levelName } from '../../product/finding-row/texts';
import { honestSize } from '../../product/sizes';
import { useFindings } from '../../state/use-findings';
import styles from './review-screen.module.css';

export type ReviewGroupSectionProps = {
  readonly group: ReviewGroup;
  readonly onToggle: (finding: Finding) => void;
};

type ListProps = { readonly label: string; readonly onToggle: (finding: Finding) => void };

// A group's list shows at most this many rows at once; it scrolls past them.
const VISIBLE_ROWS = 6;

const NOTHING_TO_LOAD = (): void => undefined;

// The list's frame, as tall as its rows (up to `VISIBLE_ROWS`).
const ListFrame = ({ children, rows }: { readonly children: JSX.Element; readonly rows: number }): JSX.Element => (
  <div className={styles.list} style={{ '--review-rows': Math.min(rows, VISIBLE_ROWS) } as CSSProperties}>
    <Surface className={styles.listSurface ?? ''} elevation={1} padding="none" radius="lg">
      {children}
    </Surface>
  </div>
);

// A selected category: all its findings, largest first, paged by the core; a finding ticked off inside it is unticked.
const CategoryRows = ({
  category,
  excluded,
  label,
  listed,
  onToggle,
}: ListProps & {
  readonly category: string;
  readonly excluded: ReadonlyMap<FindingId, Picked>;
  readonly listed: number;
}): JSX.Element => {
  const rows = useFindings({ category, module: null, order: 'largest' });

  return (
    <ListFrame rows={listed}>
      <FindingList
        failure={rows.failure}
        isSelected={(finding) => !excluded.has(finding.id)}
        label={label}
        onRangeChange={rows.onRangeChange}
        onToggle={onToggle}
        rowAt={rows.rowAt}
        // A shown group is never empty: no rows yet means its first page has not answered.
        total={rows.total === 0 && rows.failure === null ? undefined : rows.total}
      />
    </ListFrame>
  );
};

// Findings picked one by one: listed as the selection keeps them, all ticked; unticking one takes it off the list.
const PickedRows = ({
  findings,
  label,
  onToggle,
}: ListProps & { readonly findings: readonly Finding[] }): JSX.Element => (
  <ListFrame rows={findings.length}>
    <FindingList
      failure={null}
      isSelected={() => true}
      label={label}
      onRangeChange={NOTHING_TO_LOAD}
      onToggle={onToggle}
      rowAt={(index) => findings[index]}
      total={findings.length}
    />
  </ListFrame>
);

// One category of the selection: its name, highest level, count and size, what is lost if it is removed, and its rows.
export const ReviewGroupSection = ({ group, onToggle }: ReviewGroupSectionProps): JSX.Element => {
  const name = tText({ args: {}, key: `${group.category}.name` });
  const label = t('ui.screen.review.items', { category: { kind: 'text', value: name } });
  const { rows } = group;

  return (
    <section aria-label={name} className={styles.group}>
      <header className={styles.groupHeader}>
        <h2 className={styles.groupName}>{name}</h2>
        <DangerBadge label={levelName(group.highest)} level={group.highest} size="small" />
        <span className={styles.groupCount}>
          {t('ui.screen.review.count', { count: { kind: 'count', value: group.count } })}
        </span>
        <span className={styles.groupSize}>
          <SizeText {...honestSize(group.sizes)} />
        </span>
      </header>
      <p className={styles.lost}>{tText({ args: {}, key: `${group.category}.if-removed` })}</p>
      {rows.kind === 'category' ? (
        <CategoryRows
          category={group.category}
          excluded={rows.excluded}
          label={label}
          listed={rows.listed}
          onToggle={onToggle}
        />
      ) : (
        <PickedRows findings={rows.findings} label={label} onToggle={onToggle} />
      )}
    </section>
  );
};
