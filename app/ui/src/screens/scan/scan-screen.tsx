import type { JSX } from 'react';

import { Notice } from '../../design/feedback/notice/notice';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { levelShares } from '../../product/scan-hero/ring-parts';
import { ScanHero } from '../../product/scan-hero/scan-hero';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { explainedTone } from '../../product/storage-map/parts';
import { StorageMap } from '../../product/storage-map/storage-map';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useStorageMeaning } from '../../state/storage';
import { useCore, useCoreStore } from '../../state/store';
import { accessOfCategories, unavailableTotals } from '../permission/category-access';
import { CoverageNotices } from '../permission/coverage-notices';
import { useAccessStatus } from '../permission/use-access';
import { countsOf, firstWithEntry } from '../settings/ignored-tab';
import { RescanNotice } from '../settings/rescan-notice';
import { useModules } from '../use-modules';
import { useScanActions } from '../use-scan-actions';
import { LeftOutOnly } from './left-out-only';
import { moduleProgress } from './module-progress';
import { phaseOf, scanFailure } from './phase';
import styles from './scan-screen.module.css';
import { ScanZones } from './scan-zones';

// "Where did my space go?": the scan hero (the home volume's ring and legend), a notice when the scan is incomplete, the
// notices of what a missing permission kept the scan from seeing, the map of each other volume (collapsed), then
// the home volume's category cards in their danger zones, each with its Select all, and the selection bar. Before the
// first scan, only the hero, what a scan does and the other volumes.
// One control per action: the toolbar's Scan again only after a finished or partial scan; before the first scan the
// ring holds Scan, during a scan the progress holds Stop, and after a failed scan the hero's error holds Try again.
export const ScanScreen = (): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const modules = useModules();
  const { scan, stop } = useScanActions();
  const status = useCore((state) => state.status);
  const overview = useCore((state) => state.overview);
  const outcome = useCore((state) => state.outcome);
  const progress = useCore((state) => state.progress);
  const finished = useCore((state) => state.modules);
  const started = useCore((state) => state.scanModules);
  const selection = useCore((state) => state.selection);
  const storageMap = useCore((state) => state.storageMap);
  const accessStatus = useAccessStatus();
  const categoryAccess = accessOfCategories(modules, accessStatus);

  const lastScan = overview?.lastScan ?? null;
  const phase = phaseOf(status, lastScan, outcome);
  const scanning = phase === 'scanning';
  const totals = overview?.totals ?? [];
  const unavailable = unavailableTotals(modules, totals, categoryAccess);
  const ignored = overview?.ignored;
  const ignoredCount =
    ignored === undefined ? 0 : ignored.groups + ignored.modules + ignored.categories + ignored.paths;
  const volumes = storageMap?.volumes ?? [];
  const views = phase === 'scanning' || phase === 'finished' || phase === 'partial';
  // The hero's split by level comes from it.
  const meaning = useStorageMeaning(views && storageMap?.phase.kind !== 'no-scan' && volumes.length > 0 ? 0 : null);

  return (
    <>
      <Toolbar
        actions={
          phase === 'finished' || phase === 'partial' ? (
            <Button onClick={scan}>{t('ui.screen.scan.scan-again')}</Button>
          ) : undefined
        }
        title={<h1 className={styles.title}>{t('ui.screen.scan.title')}</h1>}
      />
      <div className={styles.content}>
        <ScanHero
          failure={scanFailure(lastScan, outcome)}
          levels={levelShares(meaning.groups, totals)}
          onScan={scan}
          onStop={stop}
          overview={overview}
          phase={phase}
          progress={
            scanning ? { entries: progress?.entries ?? 0, modules: moduleProgress(started, modules, finished) } : null
          }
          volume={volumes[0] ?? null}
        />
        {phase === 'idle' ? <p className={styles.intro}>{t('ui.screen.scan.intro')}</p> : null}
        {phase === 'partial' ? (
          <Notice title={t('ui.screen.scan.partial.title')} tone="warning">
            {t('ui.screen.scan.partial.text')}
          </Notice>
        ) : null}
        <RescanNotice />
        <CoverageNotices />
        {volumes.slice(1).map((volume, index) => (
          <StorageMap
            collapsible
            explained={explainedTone(totals, null)}
            key={`${volume.name}-${String(index)}`}
            scanning={scanning}
            volume={volume}
          />
        ))}
        {views && overview !== null ? (
          <ScanZones
            access={categoryAccess}
            found={overview.found}
            scanning={scanning}
            totals={totals}
            unavailable={unavailable}
          />
        ) : null}
        {views ? <LeftOutOnly cards={[...totals, ...unavailable].map(({ category }) => category)} /> : null}
        {ignoredCount > 0 ? (
          <p className={styles.ignored}>
            <span>{t('ui.screen.scan.ignored', { count: { kind: 'count', value: ignoredCount } })}</span>
            <Button
              onClick={() => {
                navigation.navigate({
                  kind: 'settings',
                  tab: ignored === undefined ? 'groups' : firstWithEntry(countsOf(ignored)),
                });
              }}
              size="small"
              variant="ghost"
            >
              {t('ui.screen.scan.ignored.show')}
            </Button>
          </p>
        ) : null}
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
