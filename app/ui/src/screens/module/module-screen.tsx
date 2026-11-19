import { type JSX, useEffect, useRef, useState } from 'react';

import type {
  CategoryDescriptor,
  CategoryId,
  CategoryTotal,
  LeftOutCount,
  ModuleOutcome,
  SettingValue,
} from '../../ipc/bindings.generated';
import type { ModuleViewProps } from '../../modules/registry';

import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { Notice } from '../../design/feedback/notice/notice';
import { CircleCheck, Minus, Search } from '../../design/icons/icons';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t, tText } from '../../i18n';
import { ModuleSection } from '../../product/module-section/module-section';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { accessOfCategories, type CategoryAccess, missingText } from '../permission/category-access';
import { useAccessStatus } from '../permission/use-access';
import { useIgnoreCommands } from '../settings/use-ignore-commands';
import { hiddenBy } from '../use-ignored';
import { moduleFound } from '../use-modules';
import { useScanActions } from '../use-scan-actions';
import { CategorySection } from './category-section';
import { ModuleChoices } from './module-choices';
import styles from './module-screen.module.css';

type Shown = { readonly category: CategoryDescriptor; readonly total: CategoryTotal };

const SCANNING = { kind: 'scanning' } as const;

const NO_LEFT_OUT: readonly LeftOutCount[] = [];

const NONE_IGNORED: readonly string[] = [];

const COMPLETED: ModuleOutcome = { kind: 'completed' };

// The ids an ignore list holds; none until the settings are loaded.
const idsOf = (value: SettingValue | undefined): readonly string[] =>
  value?.kind === 'ids' ? value.value : NONE_IGNORED;

// What a category with nothing found shows when something was left out of it.
const nothingFound = ({ danger, id }: CategoryDescriptor): CategoryTotal => ({
  category: id,
  count: 0,
  highestDanger: danger,
  sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
});

// The module page with nothing on it: an empty list, or when the module ended unavailable, why it could not look.
const NothingToShow = ({ outcome }: { readonly outcome: ModuleOutcome | undefined }): JSX.Element => {
  const status = useAccessStatus();

  return outcome?.kind !== 'unavailable' ? (
    <EmptyState icon={CircleCheck} text={t('ui.screen.module.empty.text')} title={t('ui.screen.module.empty.title')} />
  ) : (
    <EmptyState icon={Minus} text={missingText(outcome.missing, status)} title={t('ui.access.unavailable')} />
  );
};

// The module's categories that have findings, left-outs or are unavailable, largest first (ties keep the descriptor's
// order). An ignored category's findings leave the totals; its left-outs, kept from the scan's events, leave with them.
const shownOf = (
  categories: readonly CategoryDescriptor[],
  totals: readonly CategoryTotal[],
  leftOut: readonly LeftOutCount[],
  ignored: readonly string[],
  access: ReadonlyMap<CategoryId, CategoryAccess>,
): readonly Shown[] =>
  categories
    .flatMap((category) => {
      const total = totals.find((one) => one.category === category.id);

      if (total !== undefined && total.count > 0) {
        return [{ category, total }];
      }

      // An unavailable category is never silently absent.
      const left =
        !ignored.includes(category.id) &&
        (access.get(category.id)?.kind === 'unavailable' || leftOut.some((one) => one.category === category.id));

      return left ? [{ category, total: nothingFound(category) }] : [];
    })
    .sort((a, b) => b.total.sizes.allocated - a.total.sizes.allocated);

// The page every module gets for free, from its descriptor alone: what it found, one section per category with
// findings or left-outs (the largest with findings expanded), and the selection bar. Opened at a category, it expands
// it and scrolls it into view.
export const ModuleScreen = ({ category, module }: ModuleViewProps): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const { scan, stop } = useScanActions();
  const scanning = useCore((state) => state.status.state === 'scanning');
  const overview = useCore((state) => state.overview);
  const finished = useCore((state) => state.modules.find((one) => one.module === module.id));
  const outcome = finished?.outcome;
  const selection = useCore((state) => state.selection);
  // The settings are asked for by the app's frame at start; `setting-changed` events keep them current.
  const ignoredCategories = useCore((state) => state.settings?.values['core.ignore.categories']);
  const hidden = hiddenBy(overview, module.id);
  const commands = useIgnoreCommands();
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

  const leftOut = finished?.leftOut ?? NO_LEFT_OUT;
  const found = moduleFound(module, overview);
  const totals = overview?.totals ?? [];
  const categories = module.categories.map(({ descriptor }) => descriptor);
  const accessStatus = useAccessStatus();
  const access = accessOfCategories([module], accessStatus);
  // The core hides a module's findings, so an ignored module shows none of its categories or left-outs.
  const shown = hidden === undefined ? shownOf(categories, totals, leftOut, idsOf(ignoredCategories), access) : [];
  const largest = shown.find((one) => one.total.count > 0)?.category.id;
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
      />
      <PageHeader
        {...(hidden === undefined
          ? {
              actions: (
                <Tooltip content={tText({ args: {}, key: 'core.ignore.modules.description' })}>
                  {(describedBy) => (
                    <Button
                      aria-describedby={describedBy}
                      onClick={() => {
                        commands.ignore({ kind: 'module', module: module.id });
                      }}
                      size="small"
                      variant="secondary"
                    >
                      {t('ui.screen.module.ignore-module')}
                    </Button>
                  )}
                </Tooltip>
              ),
            }
          : {})}
        subtitle={tText({ args: {}, key: `${module.id}.enabled.description` })}
        title={tText({ args: {}, key: `${module.id}.enabled.label` })}
      />
      <div className={styles.content}>
        {hidden !== undefined ? (
          <Notice
            action={{
              label: t(
                hidden === 'group' ? 'ui.screen.module.ignored.restore-group' : 'ui.screen.module.ignored.restore',
              ),
              onClick: () => {
                commands.restore(
                  hidden === 'group' ? { group: module.group, kind: 'group' } : { kind: 'module', module: module.id },
                );
              },
            }}
            title={t('ui.screen.module.ignored.title')}
            tone="info"
          >
            {t('ui.screen.module.ignored.text')}
          </Notice>
        ) : found === null ? (
          <EmptyState
            icon={Search}
            text={t('ui.screen.module.not-scanned.text')}
            title={t('ui.screen.module.not-scanned.title')}
          />
        ) : (
          <ModuleSection found={found} module={module.id} nameless state={outcome ?? (scanning ? SCANNING : COMPLETED)}>
            {shown.length === 0 ? (
              <NothingToShow outcome={outcome} />
            ) : (
              <>
                <ModuleChoices module={module} totals={totals} />
                <div className={styles.categories}>
                  {shown.map((one) => (
                    <CategorySection
                      access={access.get(one.category.id)}
                      category={one.category}
                      expanded={expanded(one.category.id)}
                      key={one.category.id}
                      leftOut={leftOut.filter(({ category: id }) => id === one.category.id)}
                      module={module.id}
                      onExpandedChange={(next) => {
                        setChoices((all) => new Map(all).set(one.category.id, next));
                      }}
                      ref={one.category.id === category ? targetRef : undefined}
                      total={one.total}
                    />
                  ))}
                </div>
              </>
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
