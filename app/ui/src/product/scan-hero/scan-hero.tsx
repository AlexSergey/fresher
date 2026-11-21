import { type JSX, useState } from 'react';

import type { DangerLevel, Overview, Sizes, Text, VolumeMap } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { StorageRing, type StorageRingProps } from '../../design/data/storage-ring/storage-ring';
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
import { honestSize } from '../sizes';
import { RingLegend } from './ring-legend';
import { type RingPart, ringParts, ringSegments } from './ring-parts';
import styles from './scan-hero.module.css';
import { useScanAnnouncements } from './use-scan-announcements';

export type ScanHeroProps = {
  // The core's text of a failed scan.
  readonly failure: null | Text;
  // The cleanable bytes of the volume by danger level (`levelShares`).
  readonly levels: ReadonlyMap<DangerLevel, Sizes>;
  readonly onScan: () => void;
  readonly onStop: () => void;
  readonly overview: null | Overview;
  readonly phase: ScanPhase;
  // The gallery's: the state a pointer or a press would leave, the keys of the legend row highlighted and opened.
  readonly preview?: { readonly highlighted?: string; readonly open?: string };
  readonly progress: null | ScanProgressState;
  // The home volume the ring draws; `null` before the core has answered.
  readonly volume: null | VolumeMap;
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

type SideProps = Pick<ScanHeroProps, 'failure' | 'onScan' | 'onStop' | 'phase' | 'progress'> & {
  readonly highlighted: null | string;
  readonly onHighlight: (key: null | string) => void;
  readonly open: null | string;
  readonly parts: readonly RingPart[];
  // The volume once there is a scan to explain; `null` before it and after a failure.
  readonly volume: null | VolumeMap;
};

// Beside the ring: the legend, the disk-changed note and the scan's progress, or the failure's text and "Try again".
const Side = ({
  failure,
  highlighted,
  onHighlight,
  onScan,
  onStop,
  open,
  parts,
  phase,
  progress,
  volume,
}: SideProps): JSX.Element => (
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
        {volume === null ? null : (
          <>
            <RingLegend
              highlighted={highlighted}
              initiallyOpen={open}
              onHighlight={onHighlight}
              parts={parts}
              purgeable={volume.purgeable}
              scanning={phase === 'scanning'}
            />
            {volume.diskChanged ? (
              <p className={styles.note}>{tText({ args: {}, key: 'core.storage.changed' })}</p>
            ) : null}
          </>
        )}
        {phase === 'scanning' && progress !== null ? (
          <ScanProgress entries={progress.entries} modules={progress.modules} onStop={onStop} />
        ) : null}
      </>
    )}
  </div>
);

// The overview's hero: the home volume as one ring, the cleanable part split by danger level and Free as the gap, with
// the found size in its centre (or the Scan button before a scan), the legend beside it, and the scan's progress under
// the legend while it runs; the ambient light behind breathes during a scan. A failed scan leaves the ring empty and
// shows the core's text and "Try again" beside it.
export const ScanHero = ({
  failure,
  levels,
  onScan,
  onStop,
  overview,
  phase,
  preview,
  progress,
  volume,
}: ScanHeroProps): JSX.Element => {
  const idle = phase === 'idle';
  const known = !idle && phase !== 'failed';
  const found = known ? (overview?.found ?? NOTHING) : NOTHING;
  const items = known ? (overview?.totals ?? []).reduce((sum, { count }) => sum + count, 0) : 0;
  const categories = known ? (overview?.totals.length ?? 0) : 0;
  const parts = known && volume !== null ? ringParts(volume, levels) : [];
  const segments = ringSegments(parts);
  const [pointed, setPointed] = useState<null | string>(preview?.highlighted ?? null);
  // A key whose part is gone (the scan moved on under the pointer) points at nothing, so nothing dims.
  const highlighted = parts.some(({ key }) => key === pointed) ? pointed : null;
  const start = usePresence<HTMLDivElement>(idle, { duration: DURATION.fast, exit: FADE_OUT });
  const name = volume === null ? null : ({ kind: 'text', value: volume.name } as const);

  useScanAnnouncements(phase, found, failure);

  return (
    <Surface className={styles.hero ?? ''} elevation={1} padding="large" radius="xl">
      <AmbientLight breathing={phase === 'scanning'} />
      <div className={styles.body}>
        {volume !== null && name !== null ? (
          <p className={styles.volume}>
            {t('ui.scan-hero.caption', {
              kind: { kind: 'text', value: tText({ args: {}, key: `core.storage.kind.${volume.kind}` }) },
              name,
            })}
          </p>
        ) : null}
        <div className={styles.layout}>
          <StorageRing
            gapLabel={tText({ args: {}, key: 'core.storage.part.free.name' })}
            highlighted={highlighted}
            label={name === null ? t('ui.scan-hero.ring-unknown') : t('ui.scan-hero.ring', { name })}
            onHighlight={setPointed}
            segments={segments}
            state={RING_STATES[phase]}
            total={segments.reduce((sum, { value }) => sum + value, 0)}
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
              {known ? <Found categories={categories} found={found} items={items} /> : null}
            </div>
          </StorageRing>
          <Side
            failure={failure}
            highlighted={highlighted}
            onHighlight={setPointed}
            onScan={onScan}
            onStop={onStop}
            open={preview?.open ?? null}
            parts={parts}
            phase={phase}
            progress={progress}
            volume={known ? volume : null}
          />
        </div>
      </div>
    </Surface>
  );
};
