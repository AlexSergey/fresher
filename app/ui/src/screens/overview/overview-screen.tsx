import type { JSX } from 'react';

import type { CategoryId } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { CategoryGrid } from '../../product/category-grid/category-grid';
import { PermissionCard } from '../../product/permission-card/permission-card';
import { ScanHero } from '../../product/scan-hero/scan-hero';
import { useNavigation } from '../../state/navigation';
import { checkedOf } from '../../state/selection';
import { useCore, useCoreStore } from '../../state/store';
import { useAccess } from '../permission/use-access';
import { useModules } from '../use-modules';
import { useScanActions } from '../use-scan-actions';
import { moduleProgress } from './module-progress';
import styles from './overview-screen.module.css';
import { phaseOf, scanFailure } from './phase';

// "Where did my space go?": the scan hero, a notice when the scan is incomplete, the permission card when Full Disk
// Access is denied, and the category cards largest first. Before the first scan, only the hero and what a scan does.
// One control per action: the toolbar's Scan again only after a finished or partial scan; before the first scan the
// ring holds Scan, during a scan the progress holds Stop, and after a failed scan the hero's error holds Try again.
export const OverviewScreen = (): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const modules = useModules();
  const access = useAccess();
  const { scan, stop } = useScanActions();
  const status = useCore((state) => state.status);
  const overview = useCore((state) => state.overview);
  const outcome = useCore((state) => state.outcome);
  const progress = useCore((state) => state.progress);
  const finished = useCore((state) => state.modules);
  const started = useCore((state) => state.scanModules);
  const selection = useCore((state) => state.selection);

  const lastScan = overview?.lastScan ?? null;
  const phase = phaseOf(status, lastScan, outcome);
  const scanning = phase === 'scanning';
  const totals = overview?.totals ?? [];

  const open = (category: CategoryId): void => {
    const module = modules?.find(({ categories }) => categories.some(({ descriptor }) => descriptor.id === category));

    if (module !== undefined) {
      navigation.navigate({ category, kind: 'module', module: module.id });
    }
  };

  return (
    <>
      <Toolbar
        actions={
          phase === 'finished' || phase === 'partial' ? (
            <Button onClick={scan}>{t('ui.screen.overview.scan-again')}</Button>
          ) : undefined
        }
        title={<h1 className={styles.title}>{t('ui.screen.overview.title')}</h1>}
      />
      <div className={styles.content}>
        <ScanHero
          failure={scanFailure(lastScan, outcome)}
          onScan={scan}
          onStop={stop}
          overview={overview}
          phase={phase}
          progress={
            scanning ? { entries: progress?.entries ?? 0, modules: moduleProgress(started, modules, finished) } : null
          }
        />
        {phase === 'idle' ? <p className={styles.intro}>{t('ui.screen.overview.intro')}</p> : null}
        {phase === 'partial' ? (
          <Notice title={t('ui.screen.overview.partial.title')} tone="warning">
            {t('ui.screen.overview.partial.text')}
          </Notice>
        ) : null}
        {access === 'denied' ? <PermissionCard access="denied" /> : null}
        {phase !== 'idle' && phase !== 'failed' && overview !== null && totals.length > 0 ? (
          <CategoryGrid
            found={overview.found}
            onOpen={open}
            onToggle={(category) => {
              store.dispatch({ category, type: 'toggle-category' });
            }}
            scanning={scanning}
            selected={(category) => checkedOf(selection, category)}
            totals={totals}
          />
        ) : null}
      </div>
    </>
  );
};
