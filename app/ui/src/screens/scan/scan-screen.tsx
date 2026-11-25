import type { JSX } from 'react';

import type { Sizes, Text, VolumeMap } from '../../ipc/bindings.generated';

import { ErrorState } from '../../design/feedback/error-state/error-state';
import { Notice } from '../../design/feedback/notice/notice';
import { SectionSeparator } from '../../design/layout/section-separator/section-separator';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { DiskCard } from '../../product/disk-card/disk-card';
import { ScanningCard } from '../../product/scanning-card/scanning-card';
import { useScanAnnouncements } from '../../product/scanning-card/use-scan-announcements';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { TrashLines } from '../../product/trash-line/trash-line';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { accessOfCategories, unavailableTotals } from '../permission/category-access';
import { CoverageNotices } from '../permission/coverage-notices';
import { useAccessStatus } from '../permission/use-access';
import { countsOf, firstWithEntry } from '../settings/ignored-tab';
import { RescanNotice } from '../settings/rescan-notice';
import { useModules } from '../use-modules';
import { useScanActions } from '../use-scan-actions';
import { DescribedOnly } from './described-only';
import { LeftOutOnly } from './left-out-only';
import { phaseOf, scanFailure, type ScanPhase } from './phase';
import { lookingAt, scanPercent } from './progress';
import { ScanGroups } from './scan-groups';
import styles from './scan-screen.module.css';
import { useScannedAgo } from './use-scanned-ago';
import { openTrashLine, useTrash } from './use-trash';

type DiskProps = {
  readonly failure: null | Text;
  readonly onScan: () => void;
  readonly phase: Exclude<ScanPhase, 'scanning'>;
  readonly volume: undefined | VolumeMap;
};

// The first volume's disk card; before the core has named a volume, the Scan button alone (or the failure's text with
// Try again).
const Disk = ({ failure, onScan, phase, volume }: DiskProps): JSX.Element | null => {
  if (volume !== undefined) {
    return <DiskCard failure={failure} onScan={onScan} phase={phase} volume={volume} />;
  }
  if (phase === 'idle') {
    return (
      <div className={styles.start}>
        <Button onClick={onScan} size="large" variant="primary">
          {t('ui.scan-hero.scan')}
        </Button>
      </div>
    );
  }

  return phase === 'failed' ? (
    <ErrorState
      action={{ label: t('ui.scan-hero.try-again'), onClick: onScan }}
      text={failure === null ? '' : tText(failure)}
      title={t('ui.scan-hero.failed')}
    />
  ) : null;
};

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

// How another volume's compact card reads: it is not scanned for findings, so it holds no Scan and no failure; while a
// scan runs its sizes are lower bounds.
const OTHER_VOLUME: Readonly<Record<ScanPhase, Exclude<ScanPhase, 'failed' | 'scanning'>>> = {
  failed: 'idle',
  finished: 'finished',
  idle: 'idle',
  partial: 'partial',
  scanning: 'partial',
  stopped: 'partial',
};

// "Where did my space go?": the toolbar with when the last scan finished and Scan again, the disk card of the first
// volume (the scan's progress while it runs), a notice when the scan was stopped, what a missing permission or an
// unreadable folder kept the scan from seeing (under its own heading), what waits in the Trash, a compact card for each
// other volume, then "What you can remove": the category tiles in their groups, what was left out or only described,
// and the floating selection bar.
// One control per action: the toolbar's Scan again only after a scan that finished, ended partial or was stopped;
// before the first scan the disk card holds Scan, during a scan the progress holds Stop, and after a failed scan the
// card's error holds Try again.
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
  const trash = useTrash();
  const accessStatus = useAccessStatus();
  const categoryAccess = accessOfCategories(modules, accessStatus);

  const lastScan = overview?.lastScan ?? null;
  const phase = phaseOf(status, lastScan, outcome);
  const scanning = phase === 'scanning';
  const scanned = phase === 'finished' || phase === 'partial' || phase === 'stopped';
  const ago = useScannedAgo(scanned ? (lastScan?.finished ?? null) : null);
  const totals = overview?.totals ?? [];
  const unavailable = unavailableTotals(modules, totals, categoryAccess);
  const ignored = overview?.ignored;
  const ignoredCount =
    ignored === undefined ? 0 : ignored.groups + ignored.modules + ignored.categories + ignored.paths;
  const volumes = storageMap?.volumes ?? [];
  const views = scanning || scanned;
  const failure = scanFailure(lastScan, outcome);
  // The modules the scan runs (a scan learned of from an overview only runs every module the core lists), and which of
  // them have finished.
  const running = started ?? modules?.map(({ id }) => id) ?? [];
  const done = finished.map(({ module }) => module);
  const module = lookingAt(running, done);
  const found = overview?.found ?? NOTHING;

  useScanAnnouncements(phase, found, failure);

  return (
    <>
      <Toolbar
        actions={
          scanning ? (
            <Button onClick={stop} variant="secondary">
              {t('ui.scan-progress.stop')}
            </Button>
          ) : scanned ? (
            <>
              {ago === null ? null : (
                <span className={styles.when}>
                  {t('ui.screen.scan.scanned', { when: { kind: 'text', value: ago } })}
                </span>
              )}
              <Button onClick={scan} variant="secondary">
                {t('ui.screen.scan.scan-again')}
              </Button>
            </>
          ) : undefined
        }
        title={<h1 className={styles.title}>{t('ui.screen.scan.title')}</h1>}
      />
      <div className={styles.content}>
        {scanning ? (
          <ScanningCard
            found={found}
            module={module === null ? null : tText({ args: {}, key: `${module}.enabled.label` })}
            percent={scanPercent(progress?.bytes ?? 0, progress?.expected ?? null, {
              finished: done.length,
              started: running.length,
            })}
            volumeName={volumes[0]?.name ?? ''}
          />
        ) : (
          <Disk failure={failure} onScan={scan} phase={phase} volume={volumes[0]} />
        )}
        {phase === 'idle' ? <p className={styles.intro}>{t('ui.screen.scan.intro')}</p> : null}
        {phase === 'stopped' ? (
          <Notice title={t('ui.screen.scan.stopped.title')} tone="warning">
            {t('ui.screen.scan.stopped.text')}
          </Notice>
        ) : null}
        <RescanNotice />
        <CoverageNotices />
        {scanning ? null : <TrashLines lines={trash} onOpen={openTrashLine(store)} />}
        {volumes.slice(1).map((volume, index) => (
          <DiskCard
            compact
            failure={null}
            key={`${volume.name}-${String(index)}`}
            onScan={scan}
            phase={OTHER_VOLUME[phase]}
            volume={volume}
          />
        ))}
        {views ? <SectionSeparator label={t('ui.screen.scan.separator')} /> : null}
        {views && overview !== null ? (
          <ScanGroups access={categoryAccess} scanning={scanning} totals={totals} unavailable={unavailable} />
        ) : null}
        {views ? <LeftOutOnly cards={[...totals, ...unavailable].map(({ category }) => category)} /> : null}
        {views ? <DescribedOnly /> : null}
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
