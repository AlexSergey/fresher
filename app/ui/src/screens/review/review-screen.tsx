import { type JSX, useMemo, useRef } from 'react';

import type { CategoryId, CategoryTotal, Finding } from '../../ipc/bindings.generated';

import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { Notice } from '../../design/feedback/notice/notice';
import { ListChecks } from '../../design/icons/icons';
import { SectionSeparator } from '../../design/layout/section-separator/section-separator';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { t, tText } from '../../i18n';
import { iconOf } from '../../product/category-icon/icon-of';
import { ConfirmDialog } from '../../product/confirm-dialog/confirm-dialog';
import { confirm, discard, prepare } from '../../state/cleanup-actions';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { useAppInfo } from '../use-app-info';
import { useModules } from '../use-modules';
import { CleanupPanel } from './cleanup-panel';
import { ReviewBar } from './review-bar';
import { reviewGroups, reviewLevels } from './review-groups';
import styles from './review-screen.module.css';
import { ReviewSummary } from './review-summary';
import { ReviewTile } from './review-tile';

const NO_TOTALS: readonly CategoryTotal[] = [];

const Title = (): JSX.Element => (
  <Toolbar title={<h1 className={styles.title}>{t('ui.screen.frame.nav.review')}</h1>} />
);

// "What exactly am I about to remove?": what goes to the Trash and its groups, then one tile per category in the
// groups' order (the safest first), largest first, each with its items, every one of which can be ticked or unticked
// here; a "Look before removing" category's items go only when ticked. The bar removes through the core's check and
// the confirmation. With nothing selected, the way back to Scan.
export const ReviewScreen = (): JSX.Element => {
  const store = useCoreStore();
  const { navigate } = useNavigation();
  const modules = useModules();
  const home = useAppInfo()?.home ?? null;
  const selection = useCore((state) => state.selection);
  const totals = useCore((state) => state.overview?.totals ?? NO_TOTALS);
  const summary = useMemo(() => summarize(selection, totals), [selection, totals]);
  const groups = useMemo(() => reviewGroups(selection, totals), [selection, totals]);
  const levels = useMemo(() => reviewLevels(groups, totals), [groups, totals]);
  const removeRef = useRef<HTMLButtonElement>(null);
  const cleanup = useCore((state) => state.cleanup);

  const toggle = (finding: Finding): void => {
    store.dispatch({ finding, type: 'toggle-finding' });
  };
  // The user's own tick of all of a category, on its tile.
  const all = (category: CategoryId, ticked: boolean): void => {
    store.dispatch({ category, type: ticked ? 'tick-all' : 'untick-all' });
  };

  // A refusal of any cleanup (an uninstall confirmed from an app's page too) shows here with the core's text.
  const refusedNotice =
    cleanup.kind === 'refused' ? (
      <Notice
        action={{
          label: t('ui.screen.review.review-again'),
          onClick: () => {
            store.dispatch({ type: 'cleanup-dismissed' });
            removeRef.current?.focus();
          },
        }}
        live
        title={t('ui.screen.review.refused')}
        tone="warning"
      >
        {tText(cleanup.refusal.text)}
      </Notice>
    ) : null;

  if (cleanup.kind === 'running' || cleanup.kind === 'finished') {
    return <CleanupPanel cleanup={cleanup} />;
  }

  // Empty only when nothing goes and nothing is to check.
  if (summary.count === 0 && summary.toCheck.categories === 0) {
    return (
      <>
        <Title />
        {refusedNotice === null ? null : <div className={styles.content}>{refusedNotice}</div>}
        <div className={styles.empty}>
          <EmptyState
            action={{
              label: t('ui.screen.review.empty.action'),
              onClick: () => {
                navigate({ kind: 'scan' });
              },
            }}
            icon={ListChecks}
            text={t('ui.screen.review.empty.text')}
            title={t('ui.screen.review.empty.title')}
          />
        </div>
      </>
    );
  }

  return (
    <>
      <Title />
      <div className={styles.content}>
        {refusedNotice}
        <ReviewSummary count={summary.count} levels={levels} sizes={summary.sizes} />
        <SectionSeparator label={t('ui.screen.review.separator')} />
        {groups.map((group) => (
          <ReviewTile
            group={group}
            home={home}
            icon={iconOf(modules, group.category)}
            key={`${group.rows.kind}:${group.category}`}
            onAll={all}
            onToggle={toggle}
          />
        ))}
      </div>
      <ReviewBar
        checking={cleanup.kind === 'preparing' && cleanup.origin === 'selection'}
        count={summary.count}
        onClear={() => {
          store.dispatch({ type: 'clear' });
        }}
        onRemove={() => {
          removeRef.current?.focus();
          prepare(store);
        }}
        removeRef={removeRef}
        sizes={summary.sizes}
      />
      {cleanup.kind === 'confirming' && cleanup.origin === 'selection' ? (
        <ConfirmDialog
          onCancel={() => {
            discard(store);
          }}
          onConfirm={(method, highRiskUnderstood) => {
            confirm(store, method, highRiskUnderstood);
          }}
          open
          plan={cleanup.plan}
        />
      ) : null}
    </>
  );
};
