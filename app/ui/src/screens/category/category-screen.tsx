import { type CSSProperties, type JSX, useId } from 'react';

import type { CategoryId, LeftOutCount, ModuleInfo } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { ChevronLeft, CircleCheck } from '../../design/icons/icons';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { levelMeaning, levelName } from '../../product/finding-row/texts';
import { LeftOutLines } from '../../product/left-out-lines/left-out-lines';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { honestSize } from '../../product/sizes';
import { useNavigation } from '../../state/navigation';
import { checkedOf } from '../../state/selection';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { accessOfCategories } from '../permission/category-access';
import { useAccessStatus } from '../permission/use-access';
import { useScanActions } from '../use-scan-actions';
import { CategoryAbout } from './category-about';
import { CategoryFindings } from './category-findings';
import styles from './category-screen.module.css';
import { ModuleChoices } from './module-choices';
import { UnavailableReason } from './unavailable-reason';

export type CategoryScreenProps = {
  readonly category: CategoryId;
  // The module that lists the category, as the core describes it.
  readonly module: ModuleInfo;
};

// The findings' frame shows at most this many rows before it scrolls inside itself.
const MAX_ROWS = 12;

const NO_LEFT_OUT: readonly LeftOutCount[] = [];

const textOf = (key: string): string => tText({ args: {}, key });

// "What is this, and is it safe to remove?" for one category: its name and what it is, its level with the core's
// meaning, why it cannot be cleaned when unavailable, its four sentences, what was left out of it, the module's choices
// that touch it, and every finding with its box; the selection bar once something is selected. Back returns to the
// overview. Selecting here is not removing: Review stays the only way to a cleanup.
export const CategoryScreen = ({ category, module }: CategoryScreenProps): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const { scan, stop } = useScanActions();
  const levelId = useId();
  const itemsId = useId();
  const reasonId = useId();
  const unavailableId = useId();
  const scanning = useCore((state) => state.status.state === 'scanning');
  const overview = useCore((state) => state.overview);
  const selection = useCore((state) => state.selection);
  const leftOut = useCore((state) => state.modules.find((one) => one.module === module.id)?.leftOut) ?? NO_LEFT_OUT;
  const checked = useCore((state) => checkedOf(state.selection, category));
  const status = useAccessStatus();
  const access = accessOfCategories([module], status).get(category);
  const totals = overview?.totals ?? [];
  const total = totals.find((one) => one.category === category);
  const descriptor = module.categories.find((one) => one.descriptor.id === category)?.descriptor;
  const level = total?.highestDanger ?? descriptor?.danger;
  const unavailable = access?.kind === 'unavailable';
  const name = textOf(`${category}.name`);
  const named = { name: { kind: 'text', value: name } } as const;
  const count = total?.count ?? 0;
  const ownLeftOut = leftOut.filter((one) => one.category === category);

  return (
    <>
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
        title={
          <Button
            icon={ChevronLeft}
            onClick={() => {
              navigation.navigate({ from: category, kind: 'scan' });
            }}
            variant="ghost"
          >
            {t('ui.screen.category.back')}
          </Button>
        }
      />
      <PageHeader subtitle={textOf(`${category}.what`)} title={name} />
      <div className={styles.content}>
        {level === undefined ? null : (
          <section aria-labelledby={levelId} className={styles.part}>
            <h2 className={styles.heading} id={levelId}>
              {t('ui.screen.category.level')}
            </h2>
            <DangerBadge description={levelMeaning(level)} label={levelName(level)} level={level} size="large" />
            <p className={styles.zone}>{textOf(`core.danger.${level}.zone`)}</p>
          </section>
        )}
        {unavailable ? (
          <section aria-describedby={reasonId} aria-labelledby={unavailableId} className={styles.part}>
            <h2 className={styles.heading} id={unavailableId}>
              {t('ui.access.unavailable')}
            </h2>
            <UnavailableReason id={reasonId} missing={access.missing} />
          </section>
        ) : null}
        <CategoryAbout category={category} />
        <LeftOutLines counts={ownLeftOut} />
        <ModuleChoices category={category} module={module} totals={totals} />
        {unavailable || (scanning && count === 0) ? null : total === undefined || count === 0 ? (
          ownLeftOut.length > 0 ? (
            <p className={styles.note}>{t('ui.screen.category.empty.left-out')}</p>
          ) : (
            <EmptyState
              icon={CircleCheck}
              text={t('ui.screen.category.empty.text')}
              title={t('ui.screen.category.empty.title')}
            />
          )
        ) : (
          <section aria-labelledby={itemsId} className={styles.part}>
            <div className={styles.head}>
              <Checkbox
                checked={checked}
                label={t('ui.screen.category.select', named)}
                onChange={() => {
                  store.dispatch({ category, type: 'toggle-category' });
                }}
              />
              <span className={styles.count} id={itemsId}>
                {t('ui.screen.category.count', { count: { kind: 'count', value: count } })}
              </span>
              <SizeText {...honestSize(total.sizes)} />
            </div>
            {access?.kind === 'limited' ? <p className={styles.note}>{access.text}</p> : null}
            <div className={styles.findings} style={{ '--rows': Math.min(count, MAX_ROWS) } as CSSProperties}>
              <CategoryFindings category={category} module={module.id} name={name} />
            </div>
          </section>
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
    </>
  );
};
