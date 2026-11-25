import { type CSSProperties, type JSX, useId, useLayoutEffect, useRef } from 'react';

import type { CategoryId, LeftOutCount, ModuleInfo } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { ChevronLeft, CircleCheck } from '../../design/icons/icons';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { t, tText } from '../../i18n';
import { BY_HAND } from '../../product/danger-zone/zones';
import { LeftOutLines } from '../../product/left-out-lines/left-out-lines';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { honestSize } from '../../product/sizes';
import { useNavigation } from '../../state/navigation';
import { checkedOf } from '../../state/selection';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { accessOfCategories } from '../permission/category-access';
import { useAccessStatus } from '../permission/use-access';
import { useAppInfo } from '../use-app-info';
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

// "What is this, and is it safe to remove?" for one category: its name, its group, its size and count, why it cannot
// be cleaned when unavailable, the three answers, what was left out of it as bars, the module's choices that touch
// it, and every finding with its box (and one "Select all" for the whole category, unless it is "Look before
// removing"); the selection bar once something is selected. Back returns to the overview. Selecting here is not
// removing: Review stays the only way to a cleanup.
export const CategoryScreen = ({ category, module }: CategoryScreenProps): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const reasonId = useId();
  const unavailableId = useId();
  const scanning = useCore((state) => state.status.state === 'scanning');
  const overview = useCore((state) => state.overview);
  const selection = useCore((state) => state.selection);
  const leftOut = useCore((state) => state.modules.find((one) => one.module === module.id)?.leftOut) ?? NO_LEFT_OUT;
  const checked = useCore((state) => checkedOf(state.selection, category));
  const home = useAppInfo()?.home ?? null;
  const status = useAccessStatus();
  const access = accessOfCategories([module], status).get(category);
  const totals = overview?.totals ?? [];
  const total = totals.find((one) => one.category === category);
  const descriptor = module.categories.find((one) => one.descriptor.id === category)?.descriptor;
  const level = total?.highestDanger ?? descriptor?.danger;
  const unavailable = access?.kind === 'unavailable';
  // A "Look before removing" category is never selected whole here: its rows are ticked one by one.
  const byHand = level === undefined || BY_HAND.has(level);
  const name = textOf(`${category}.name`);
  const named = { name: { kind: 'text', value: name } } as const;
  const count = total?.count ?? 0;
  const ownLeftOut = leftOut.filter((one) => one.category === category);
  const backRef = useRef<HTMLButtonElement>(null);
  const previousCountRef = useRef(count);

  // The last row left (it was ignored, not a scan starting): the focus fell to the page, so it goes to the screen's
  // first control.
  useLayoutEffect(() => {
    const focused = document.activeElement;

    if (!scanning && previousCountRef.current > 0 && count === 0 && (focused === null || focused === document.body)) {
      backRef.current?.focus();
    }
    previousCountRef.current = count;
  }, [count, scanning]);

  return (
    <>
      <Toolbar
        title={
          <Button
            aria-label={t('ui.screen.category.back-label')}
            icon={ChevronLeft}
            onClick={() => {
              navigation.navigate({ from: category, kind: 'scan' });
            }}
            ref={backRef}
            variant="ghost"
          >
            {t('ui.screen.category.back')}
          </Button>
        }
      />
      <header className={styles.header}>
        <div className={styles.name}>
          <h1 className={styles.title}>{name}</h1>
          {level === undefined ? null : (
            <span className={styles.group}>
              <span className={styles.dot} data-level={level} />
              {textOf(`core.danger.${level}.zone`)}
            </span>
          )}
        </div>
        {total === undefined ? null : (
          <div className={styles.total}>
            <SizeText {...honestSize(total.sizes)} size="title-1" />
            <span className={styles.count}>
              {t('ui.screen.category.count', { count: { kind: 'count', value: count } })}
            </span>
          </div>
        )}
      </header>
      <div className={styles.content}>
        {unavailable ? (
          <section aria-describedby={reasonId} aria-labelledby={unavailableId} className={styles.part}>
            <h2 className={styles.heading} id={unavailableId}>
              {t('ui.access.unavailable')}
            </h2>
            <UnavailableReason id={reasonId} missing={access.missing} />
          </section>
        ) : null}
        <CategoryAbout category={category} />
        <LeftOutLines bar counts={ownLeftOut} />
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
          <section aria-label={t('ui.screen.category.items', named)} className={styles.part}>
            {byHand ? null : (
              <div className={styles.selectAll}>
                <Checkbox
                  checked={checked}
                  label={t('ui.screen.category.select-all', { count: { kind: 'count', value: count } })}
                  onChange={() => {
                    store.dispatch({ category, type: 'toggle-category' });
                  }}
                />
              </div>
            )}
            {access?.kind === 'limited' ? <p className={styles.note}>{access.text}</p> : null}
            <div className={styles.findings} style={{ '--rows': Math.min(count, MAX_ROWS) } as CSSProperties}>
              <CategoryFindings category={category} home={home} module={module.id} name={name} />
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
