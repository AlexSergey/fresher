import { type JSX, useMemo } from 'react';

import type { CategoryTotal, DangerLevel, Finding, Sizes } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { EmptyState } from '../../design/feedback/empty-state/empty-state';
import { ListChecks, Trash2 } from '../../design/icons/icons';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { t } from '../../i18n';
import { levelMeaning, levelName } from '../../product/finding-row/texts';
import { honestSize } from '../../product/sizes';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { ReviewGroupSection } from './review-group';
import { reviewGroups } from './review-groups';
import styles from './review-screen.module.css';

type SummaryProps = { readonly count: number; readonly highest: DangerLevel; readonly sizes: Sizes };

const NO_TOTALS: readonly CategoryTotal[] = [];

// What the selection adds up to (the selection bar's preview), and the removal, which waits for plan 008: the button
// stays disabled and says so. The confirmation will show only the core's cleanup plan, never these sums.
const Summary = ({ count, highest, sizes }: SummaryProps): JSX.Element => (
  <section aria-label={t('ui.screen.review.summary')}>
    <Surface className={styles.summary ?? ''} elevation={1} padding="large" radius="lg">
      <div className={styles.figures}>
        <span className={styles.count}>{t('ui.screen.review.count', { count: { kind: 'count', value: count } })}</span>
        <SizeText {...honestSize(sizes)} size="display" />
      </div>
      <div className={styles.level}>
        <VisuallyHidden>{t('ui.screen.review.highest')}</VisuallyHidden>
        <DangerBadge description={levelMeaning(highest)} label={levelName(highest)} level={highest} size="large" />
      </div>
      <div className={styles.remove}>
        <Button disabled icon={Trash2} size="large" variant="primary">
          {t('ui.screen.review.remove')}
        </Button>
        <p className={styles.later}>{t('ui.screen.review.remove-later')}</p>
      </div>
    </Surface>
  </section>
);

// "What exactly am I about to remove?": the selection's summary, then the selection grouped by category, largest
// first, each with what it costs; every row can be unticked here. With nothing selected, the way back to the overview.
export const ReviewScreen = (): JSX.Element => {
  const store = useCoreStore();
  const { navigate } = useNavigation();
  const selection = useCore((state) => state.selection);
  const totals = useCore((state) => state.overview?.totals ?? NO_TOTALS);
  const summary = useMemo(() => summarize(selection, totals), [selection, totals]);
  const groups = useMemo(() => reviewGroups(selection, totals), [selection, totals]);
  const { highest } = summary;

  const toggle = (finding: Finding): void => {
    store.dispatch({ finding, type: 'toggle-finding' });
  };

  if (highest === null || summary.count === 0) {
    return (
      <>
        <Toolbar />
        <PageHeader title={t('ui.screen.frame.nav.review')} />
        <div className={styles.empty}>
          <EmptyState
            action={{
              label: t('ui.screen.review.empty.action'),
              onClick: () => {
                navigate({ kind: 'overview' });
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
      <Toolbar
        actions={
          <Button
            onClick={() => {
              store.dispatch({ type: 'clear' });
            }}
            variant="ghost"
          >
            {t('ui.screen.review.clear')}
          </Button>
        }
      />
      <PageHeader subtitle={t('ui.screen.review.subtitle')} title={t('ui.screen.frame.nav.review')} />
      <div className={styles.content}>
        <Summary count={summary.count} highest={highest} sizes={summary.sizes} />
        {groups.map((group) => (
          <ReviewGroupSection group={group} key={`${group.rows.kind}:${group.category}`} onToggle={toggle} />
        ))}
      </div>
    </>
  );
};
