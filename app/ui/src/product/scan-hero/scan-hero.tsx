import { type JSX, useState } from 'react';

import type { Overview, Sizes, Text } from '../../ipc/bindings.generated';

import { AnimatedNumber } from '../../design/data/animated-number/animated-number';
import { SizeText } from '../../design/data/size-text/size-text';
import { StatTile } from '../../design/data/stat-tile/stat-tile';
import { StorageRing, type StorageRingProps } from '../../design/data/storage-ring/storage-ring';
import { UsageBar } from '../../design/data/usage-bar/usage-bar';
import { AmbientLight } from '../../design/effects/ambient-light/ambient-light';
import { ErrorState } from '../../design/feedback/error-state/error-state';
import { Surface } from '../../design/layout/surface/surface';
import { useAnimatedNumber, usePresence } from '../../design/motion';
import { Button } from '../../design/primitives/button/button';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { DURATION } from '../../design/tokens/motion';
import { t, tText } from '../../i18n';
import { formatBytes } from '../../i18n/format-size';
import { type ModuleProgress, ScanProgress } from '../scan-progress/scan-progress';
import { formatCount, honestSize, sizeLabel } from '../sizes';
import styles from './scan-hero.module.css';
import { useScanAnnouncements } from './use-scan-announcements';

export type ScanHeroProps = {
  // The core's text of a failed scan.
  readonly failure: null | Text;
  readonly onScan: () => void;
  readonly onStop: () => void;
  readonly overview: null | Overview;
  readonly phase: ScanPhase;
  readonly progress: null | ScanProgressState;
};

// Where the scan is, as the screen derives it from the core's status and the last scan's outcome.
export type ScanPhase = 'failed' | 'finished' | 'idle' | 'partial' | 'scanning';

// What the running scan reported: how many items it looked at and where each module is.
type ScanProgressState = {
  readonly entries: number;
  readonly modules: readonly ModuleProgress[];
};

const RING_STATES: Readonly<Record<ScanPhase, StorageRingProps['state']>> = {
  failed: 'empty',
  finished: 'done',
  idle: 'empty',
  partial: 'partial',
  scanning: 'scanning',
};

const NOTHING: Sizes = { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 };

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

const nameOf = (category: string): string => tText({ args: {}, key: `${category}.name` });

// The number and its unit, the unit smaller beside it, so the hero size fits inside the ring.
const HeroCount = ({ duration, value }: { readonly duration: number; readonly value: number }): JSX.Element => {
  const [, number = '', unit = ''] = /^(\S+)\s(.*)$/u.exec(formatBytes(useAnimatedNumber(value, { duration }))) ?? [];

  return (
    <span aria-hidden="true" className={styles.count} data-numeric>
      {number}
      <span className={styles.unit}>{unit}</span>
    </span>
  );
};

type FoundProps = {
  readonly categories: number;
  readonly found: Sizes;
  readonly items: number;
};

// The centre of a scan: "Found", the counting size, and how many items and categories hold it. The first fill counts
// deliberately, later totals retarget slowly; a size that falls (a new scan) starts again from where it is.
const Found = ({ categories, found, items }: FoundProps): JSX.Element => {
  const value = found.allocated;
  const [run, setRun] = useState<{ readonly first: null | number; readonly generation: number; readonly last: number }>(
    {
      first: value > 0 ? value : null,
      generation: 0,
      last: value,
    },
  );

  if (value !== run.last) {
    const fell = value < run.last;

    setRun({
      first: fell || run.first === null ? (value > 0 ? value : null) : run.first,
      generation: fell ? run.generation + 1 : run.generation,
      last: value,
    });
  }

  const duration = run.first === null || run.first === value ? DURATION.deliberate : DURATION.slow;
  const honest = honestSize(found);

  return (
    <div className={styles.found}>
      <span className={styles.caption}>{t('ui.scan-hero.found')}</span>
      {found.completeness === 'unknown' ? (
        <SizeText {...honest} size="hero" />
      ) : (
        <span className={styles.number}>
          <HeroCount duration={duration} key={run.generation} value={value} />
          {found.completeness === 'partial' ? (
            <span aria-hidden="true" className={styles.plus}>
              +
            </span>
          ) : null}
          <VisuallyHidden>{honest.accessibleText}</VisuallyHidden>
        </span>
      )}
      <span className={styles.summary}>
        {t('ui.scan-hero.summary', {
          categories: {
            kind: 'text',
            value: t('ui.scan-hero.categories', { count: { kind: 'count', value: categories } }),
          },
          items: { kind: 'text', value: t('ui.scan-hero.items', { count: { kind: 'count', value: items } }) },
        })}
      </span>
    </div>
  );
};

// A count that grows with the scan, or a dash before there is one.
const Count = ({ known, value }: { readonly known: boolean; readonly value: number }): JSX.Element =>
  known ? (
    <AnimatedNumber format={formatCount} value={value} />
  ) : (
    <span data-numeric>
      <span aria-hidden="true">—</span>
      <VisuallyHidden>{t('ui.scan-hero.stat.none')}</VisuallyHidden>
    </span>
  );

// The overview's hero: the storage ring with the found size in its centre (or the Scan button before a scan), the
// stat tiles and the usage bar beside it, and the scan's progress under them while it runs; the ambient light behind
// breathes during a scan. A failed scan leaves the ring empty and shows the core's text and "Try again" beside it.
export const ScanHero = ({ failure, onScan, onStop, overview, phase, progress }: ScanHeroProps): JSX.Element => {
  const idle = phase === 'idle';
  const known = !idle && phase !== 'failed';
  const found = known ? (overview?.found ?? NOTHING) : NOTHING;
  const totals = known ? (overview?.totals ?? []) : [];
  const items = totals.reduce((sum, { count }) => sum + count, 0);
  const segments = totals.map(({ category, highestDanger, sizes }) => ({
    key: category,
    label: nameOf(category),
    size: sizeLabel(sizes),
    tone: highestDanger,
    value: sizes.allocated,
  }));
  const start = usePresence<HTMLDivElement>(idle, { duration: DURATION.fast, exit: FADE_OUT });

  useScanAnnouncements(phase, found, failure);

  return (
    <Surface className={styles.hero ?? ''} elevation={1} padding="large" radius="xl">
      <AmbientLight breathing={phase === 'scanning'} />
      <div className={styles.layout}>
        <StorageRing
          label={t('ui.scan-hero.ring')}
          segments={segments}
          state={RING_STATES[phase]}
          total={found.allocated}
        >
          <div className={styles.centre}>
            {start.mounted ? (
              <div className={styles.start} inert={!idle} ref={start.ref}>
                <span aria-hidden="true" className={styles.glow} />
                <Button onClick={onScan} size="large" variant="primary">
                  {t('ui.scan-hero.scan')}
                </Button>
              </div>
            ) : null}
            {known ? <Found categories={totals.length} found={found} items={items} /> : null}
          </div>
        </StorageRing>
        <div className={styles.side}>
          {phase === 'failed' ? (
            <div className={styles.failed}>
              <ErrorState
                action={{ label: t('ui.scan-hero.try-again'), onClick: onScan }}
                text={failure === null ? '' : tText(failure)}
                title={t('ui.scan-hero.failed')}
              />
            </div>
          ) : (
            <>
              <div className={styles.stats}>
                <StatTile
                  label={t('ui.scan-hero.found')}
                  value={known ? <SizeText {...honestSize(found)} size="title-3" /> : <Count known={false} value={0} />}
                />
                <StatTile label={t('ui.scan-hero.stat.items')} value={<Count known={known} value={items} />} />
                <StatTile
                  label={t('ui.scan-hero.stat.categories')}
                  value={<Count known={known} value={totals.length} />}
                />
              </div>
              <UsageBar label={t('ui.scan-hero.usage')} legend segments={segments} total={found.allocated} />
              {phase === 'scanning' && progress !== null ? (
                <ScanProgress entries={progress.entries} modules={progress.modules} onStop={onStop} />
              ) : null}
            </>
          )}
        </div>
      </div>
    </Surface>
  );
};
