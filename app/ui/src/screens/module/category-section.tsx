import type { CSSProperties, JSX, Ref } from 'react';

import type { CategoryDescriptor, CategoryId, CategoryTotal, ModuleId } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { TreeRow } from '../../design/data/tree-row/tree-row';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { FindingList } from '../../product/finding-list/finding-list';
import { levelName } from '../../product/finding-row/texts';
import { honestSize } from '../../product/sizes';
import { checkedOf } from '../../state/selection';
import { useCore, useCoreStore } from '../../state/store';
import { useFindings } from '../../state/use-findings';
import styles from './module-screen.module.css';

export type CategorySectionProps = {
  readonly category: CategoryDescriptor;
  readonly expanded: boolean;
  readonly module: ModuleId;
  readonly onExpandedChange: (expanded: boolean) => void;
  // The page scrolls to the section it was opened at.
  readonly ref?: Ref<HTMLElement> | undefined;
  readonly total: CategoryTotal;
};

// An expanded list shows at most this many rows before it scrolls inside itself.
const MAX_ROWS = 8;

const textOf = (key: string): string => tText({ args: {}, key });

// The findings of an expanded category, largest first; mounted only when expanded, so a collapsed one asks for none.
const CategoryFindings = ({
  category,
  module,
  name,
}: {
  readonly category: CategoryId;
  readonly module: ModuleId;
  readonly name: string;
}): JSX.Element => {
  const store = useCoreStore();
  const selection = useCore((state) => state.selection);
  const { failure, onRangeChange, rowAt, total } = useFindings({ category, module, order: 'largest' });

  return (
    <FindingList
      failure={failure}
      isSelected={(finding) => {
        const excluded = selection.categories.get(finding.category);

        return excluded === undefined ? selection.findings.has(finding.id) : !excluded.has(finding.id);
      }}
      label={t('ui.screen.module.items', { name: { kind: 'text', value: name } })}
      onRangeChange={onRangeChange}
      onToggle={(finding) => {
        store.dispatch({ finding, type: 'toggle-finding' });
      }}
      rowAt={rowAt}
      total={total}
    />
  );
};

// What the category is, in the core's four plain sentences, behind a disclosure.
const About = ({ category }: { readonly category: CategoryId }): JSX.Element => (
  <details className={styles.about}>
    <summary className={styles.summary}>{t('ui.screen.module.about')}</summary>
    <dl className={styles.sentences}>
      <dt>{t('ui.screen.module.what')}</dt>
      <dd>{textOf(`${category}.what`)}</dd>
      <dt>{t('ui.screen.module.made-by')}</dt>
      <dd>{textOf(`${category}.made-by`)}</dd>
      <dt>{t('ui.screen.module.if-removed')}</dt>
      <dd>{textOf(`${category}.if-removed`)}</dd>
      <dt>{t('ui.screen.module.comes-back')}</dt>
      <dd>{textOf(`${category}.comes-back`)}</dd>
    </dl>
  </details>
);

// One category of a module's page: its row (name, level, size, count, its box), what it is, and its findings when
// expanded.
export const CategorySection = ({
  category,
  expanded,
  module,
  onExpandedChange,
  ref,
  total,
}: CategorySectionProps): JSX.Element => {
  const store = useCoreStore();
  const checked = useCore((state) => checkedOf(state.selection, category.id));
  const name = textOf(`${category.id}.name`);
  const named = { name: { kind: 'text', value: name } } as const;

  return (
    <section aria-label={name} className={styles.category} ref={ref}>
      <div role="list">
        <TreeRow
          expanded={expanded}
          leading={
            <Checkbox
              checked={checked}
              hideLabel
              label={t('ui.screen.module.select', named)}
              onChange={() => {
                store.dispatch({ category: category.id, type: 'toggle-category' });
              }}
            />
          }
          onExpandedChange={onExpandedChange}
          subtitle={t('ui.screen.module.count', { count: { kind: 'count', value: total.count } })}
          title={name}
          toggleLabel={t('ui.screen.module.expand', named)}
          trailing={
            <>
              <DangerBadge label={levelName(category.danger)} level={category.danger} />
              <SizeText {...honestSize(total.sizes)} />
            </>
          }
        />
      </div>
      <About category={category.id} />
      {expanded ? (
        <div
          className={styles.findings}
          style={{ '--rows': Math.min(Math.max(total.count, 1), MAX_ROWS) } as CSSProperties}
        >
          <CategoryFindings category={category.id} module={module} name={name} />
        </div>
      ) : null}
    </section>
  );
};
