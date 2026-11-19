import type { JSX } from 'react';

import type { CategoryId } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { SegmentedControl } from '../../design/primitives/segmented-control/segmented-control';
import { t } from '../../i18n';
import { MeaningList } from '../../product/meaning-list/meaning-list';
import { ScanHero } from '../../product/scan-hero/scan-hero';
import { explainedTone } from '../../product/storage-map/parts';
import { StorageMap } from '../../product/storage-map/storage-map';
import { placeTop, useNavigation, useOverviewView } from '../../state/navigation';
import { checkedOf } from '../../state/selection';
import { useStorageMeaning } from '../../state/storage';
import { useCore, useCoreStore } from '../../state/store';
import { accessOfCategories, unavailableTotals } from '../permission/category-access';
import { CoverageNotices } from '../permission/coverage-notices';
import { useAccessStatus } from '../permission/use-access';
import { countsOf, firstWithEntry } from '../settings/ignored-tab';
import { RescanNotice } from '../settings/rescan-notice';
import { useModules } from '../use-modules';
import { useScanActions } from '../use-scan-actions';
import { ByPlace } from './by-place';
import { moduleProgress } from './module-progress';
import styles from './overview-screen.module.css';
import { phaseOf, scanFailure } from './phase';

// "Where did my space go?": the scan hero, a notice when the scan is incomplete, the notices of what a missing permission
// kept the scan from seeing, the map of each volume (the home's first, any other collapsed and shown by its map only), then
// the switch between the views: the home volume's view by meaning, its category cards largest first, or its view by
// place, its folders largest first. Before the first scan, only the hero, what a scan does and the volume.
// One control per action: the toolbar's Scan again only after a finished or partial scan; before the first scan the
// ring holds Scan, during a scan the progress holds Stop, and after a failed scan the hero's error holds Try again.
export const OverviewScreen = (): JSX.Element => {
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
  const view = useOverviewView();
  const accessStatus = useAccessStatus();
  const categoryAccess = accessOfCategories(modules, accessStatus);

  const lastScan = overview?.lastScan ?? null;
  const phase = phaseOf(status, lastScan, outcome);
  const scanning = phase === 'scanning';
  const totals = overview?.totals ?? [];
  const ignored = overview?.ignored;
  const ignoredCount =
    ignored === undefined ? 0 : ignored.groups + ignored.modules + ignored.categories + ignored.paths;
  const volumes = storageMap?.volumes ?? [];
  const views = phase === 'scanning' || phase === 'finished' || phase === 'partial';
  // Asked in both views: the bar's tone of Explained comes from it, so switching the view changes nothing above it.
  const meaning = useStorageMeaning(views && storageMap?.phase.kind !== 'no-scan' && volumes.length > 0 ? 0 : null);

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
        <RescanNotice />
        <CoverageNotices />
        {volumes.map((volume, index) => (
          <StorageMap
            collapsible={index > 0}
            explained={index === 0 ? explainedTone(totals, meaning.groups) : 'plain'}
            key={`${volume.name}-${String(index)}`}
            scanning={scanning}
            volume={volume}
          />
        ))}
        {views && overview !== null ? (
          <>
            <div className={styles.views}>
              <SegmentedControl
                label={t('ui.screen.overview.view.label')}
                onChange={(kind) => {
                  navigation.showOverviewView(kind === 'meaning' ? { kind } : placeTop(0));
                }}
                options={[
                  { label: t('ui.screen.overview.view.meaning'), value: 'meaning' },
                  { label: t('ui.screen.overview.view.place'), value: 'place' },
                ]}
                value={view.kind}
              />
            </div>
            {view.kind === 'meaning' ? (
              <MeaningList
                access={categoryAccess}
                failure={meaning.failure}
                found={overview.found}
                groups={meaning.groups}
                onOpen={open}
                onOpenPlace={(node) => {
                  navigation.showOverviewView({ ...placeTop(0), node });
                }}
                onToggle={(category) => {
                  store.dispatch({ category, type: 'toggle-category' });
                }}
                scanning={scanning}
                selected={(category) => checkedOf(selection, category)}
                totals={totals}
                unavailable={unavailableTotals(modules, totals, categoryAccess)}
              />
            ) : (
              <ByPlace at={view} />
            )}
          </>
        ) : null}
        {ignoredCount > 0 ? (
          <p className={styles.ignored}>
            <span>{t('ui.screen.overview.ignored', { count: { kind: 'count', value: ignoredCount } })}</span>
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
              {t('ui.screen.overview.ignored.show')}
            </Button>
          </p>
        ) : null}
      </div>
    </>
  );
};
