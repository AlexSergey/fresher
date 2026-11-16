import { type JSX, useEffect, useRef, useState } from 'react';

import type { CategoryDescriptor, CategoryId, CategoryTotal, ModuleOutcome } from '../../ipc/bindings.generated';
import type { ModuleViewProps } from '../../modules/registry';

import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { CircleCheck, Search } from '../../design/icons/icons';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { ModuleSection } from '../../product/module-section/module-section';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { moduleFound } from '../use-modules';
import { useScanActions } from '../use-scan-actions';
import { CategorySection } from './category-section';
import styles from './module-screen.module.css';

type Shown = { readonly category: CategoryDescriptor; readonly total: CategoryTotal };

const SCANNING = { kind: 'scanning' } as const;

const COMPLETED: ModuleOutcome = { kind: 'completed' };

// The module's categories that have findings, largest first (ties keep the descriptor's order).
const shownOf = (categories: readonly CategoryDescriptor[], totals: readonly CategoryTotal[]): readonly Shown[] =>
  categories
    .flatMap((category) => {
      const total = totals.find((one) => one.category === category.id);

      return total === undefined || total.count === 0 ? [] : [{ category, total }];
    })
    .sort((a, b) => b.total.sizes.allocated - a.total.sizes.allocated);

// The page every module gets for free, from its descriptor alone: what it found, one section per category with
// findings (the largest expanded), and the selection bar. Opened at a category, it expands it and scrolls it into view.
export const ModuleScreen = ({ category, module }: ModuleViewProps): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const { scan, stop } = useScanActions();
  const scanning = useCore((state) => state.status.state === 'scanning');
  const overview = useCore((state) => state.overview);
  const outcome = useCore((state) => state.modules.find((one) => one.module === module.id)?.outcome);
  const selection = useCore((state) => state.selection);
  // The user's own choices; a category not in it is expanded when it is the largest or the one the page opened at.
  const [choices, setChoices] = useState<ReadonlyMap<CategoryId, boolean>>(() => new Map());
  const [openedAt, setOpenedAt] = useState(category);
  const targetRef = useRef<HTMLElement>(null);
  const scrolledToRef = useRef<CategoryId | undefined>(undefined);

  if (openedAt !== category) {
    setOpenedAt(category);
    if (category !== undefined) {
      setChoices((all) => new Map(all).set(category, true));
    }
  }

  const found = moduleFound(module, overview);
  const totals = overview?.totals ?? [];
  const shown = shownOf(module.categories, totals);
  const largest = shown[0]?.category.id;
  const present = category !== undefined && shown.some((one) => one.category.id === category);

  useEffect(() => {
    if (present && scrolledToRef.current !== category) {
      scrolledToRef.current = category;
      targetRef.current?.scrollIntoView({ block: 'start' });
    }
  }, [category, present]);

  const expanded = (id: CategoryId): boolean => choices.get(id) ?? (id === largest || id === category);

  return (
    <div className={styles.page}>
      <Toolbar
        actions={
          scanning ? (
            <Button onClick={stop}>{t('ui.scan-progress.stop')}</Button>
          ) : (
            <Button onClick={scan} variant="primary">
              {t('ui.scan-hero.scan')}
            </Button>
          )
        }
        title={<h1 className={styles.title}>{tText({ args: {}, key: `${module.id}.enabled.label` })}</h1>}
      />
      <div className={styles.content}>
        {found === null ? (
          <EmptyState
            icon={Search}
            text={t('ui.screen.module.not-scanned.text')}
            title={t('ui.screen.module.not-scanned.title')}
          />
        ) : (
          <ModuleSection found={found} module={module.id} state={outcome ?? (scanning ? SCANNING : COMPLETED)}>
            {shown.length === 0 ? (
              <EmptyState
                icon={CircleCheck}
                text={t('ui.screen.module.empty.text')}
                title={t('ui.screen.module.empty.title')}
              />
            ) : (
              <div className={styles.categories}>
                {shown.map((one) => (
                  <CategorySection
                    category={one.category}
                    expanded={expanded(one.category.id)}
                    key={one.category.id}
                    module={module.id}
                    onExpandedChange={(next) => {
                      setChoices((all) => new Map(all).set(one.category.id, next));
                    }}
                    ref={one.category.id === category ? targetRef : undefined}
                    total={one.total}
                  />
                ))}
              </div>
            )}
          </ModuleSection>
        )}
      </div>
      <SelectionBar
        onClear={() => {
          store.dispatch({ type: 'clear' });
        }}
        onReview={() => {
          navigation.navigate({ kind: 'review' });
        }}
        summary={summarize(selection, totals)}
      />
    </div>
  );
};
