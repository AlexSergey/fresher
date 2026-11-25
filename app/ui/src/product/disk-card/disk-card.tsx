import { type CSSProperties, type JSX, useState } from 'react';

import type { Sizes, SummaryKind, Text, VolumeMap } from '../../ipc/bindings.generated';

import { type RingLook, StorageRing, type StorageRingProps } from '../../design/data/storage-ring/storage-ring';
import { ErrorState } from '../../design/feedback/error-state/error-state';
import { Button } from '../../design/primitives/button/button';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { t, tText } from '../../i18n';
import { formatBytes } from '../../i18n/format-size';
import { honestSize } from '../sizes';
import styles from './disk-card.module.css';
import { DiskLegend } from './disk-legend';
import { diskParts, partsText, ringSegments } from './disk-parts';

export type DiskCardProps = {
  // Smaller: the ring and its centre shrink, the legend stays.
  readonly compact?: boolean;
  // The core's text of a failed scan.
  readonly failure: null | Text;
  readonly onScan: () => void;
  readonly phase: DiskCardPhase;
  // The gallery's: the state a pointer or a press would leave, the kind of the legend row highlighted and opened.
  readonly preview?: { readonly highlighted?: SummaryKind; readonly open?: SummaryKind };
  readonly volume: VolumeMap;
};

// The screen's `ScanPhase` without `scanning`, which is the scan card's.
type DiskCardPhase = 'failed' | 'finished' | 'idle' | 'partial';

const RING_STATES: Readonly<Record<DiskCardPhase, StorageRingProps['state']>> = {
  failed: 'empty',
  finished: 'done',
  idle: 'empty',
  partial: 'partial',
};

// The ring of the design: a gap of 3 and a least arc of 6 units of its 465-unit circle, square ends, no glow.
const UNIT = 360 / 465;
const LOOK: RingLook = { caps: 'butt', gap: 3 * UNIT, glow: false, minSweep: 6 * UNIT };

const SIZES = { card: { diameter: 180, thickness: 16 }, compact: { diameter: 96, thickness: 10 } } as const;

// The size under the ring's centre: the number big, read by its honest text.
const CentreSize = ({ sizes }: { readonly sizes: Sizes }): JSX.Element => {
  const { accessibleText, completeness, text } = honestSize(sizes);

  return (
    <>
      <span aria-hidden="true" className={styles.count} data-numeric>
        {completeness === 'unknown' ? '—' : text}
        {completeness === 'partial' ? '+' : null}
      </span>
      <VisuallyHidden>{accessibleText}</VisuallyHidden>
    </>
  );
};

// "109 GB free of 494 GB", the free size bold; the dictionary's order of the words is kept. A line that does not hold
// the free size as written is shown plain.
const FreeLine = ({ capacity, free }: { readonly capacity: number; readonly free: string }): JSX.Element => {
  const line = t('ui.disk-card.free-of', {
    capacity: { kind: 'text', value: formatBytes(capacity) },
    free: { kind: 'text', value: free },
  });
  const at = line.indexOf(free);

  if (at < 0) {
    return <span className={styles.free}>{line}</span>;
  }

  return (
    <span className={styles.free}>
      {line.slice(0, at)}
      <b>{free}</b>
      {line.slice(at + free.length)}
    </span>
  );
};

// The card of one volume: a ring of its five parts with what can be freed in its centre (the Scan button before the
// first scan, on the full card only), its name and free space, and a legend of the parts, each with an "i". A compact
// card, another volume's, is named by its volume. A row and its arc highlight
// each other. A failed scan leaves the ring empty and shows the core's text and "Try again" in the legend's place.
export const DiskCard = ({ compact = false, failure, onScan, phase, preview, volume }: DiskCardProps): JSX.Element => {
  const parts = diskParts(volume);
  const segments = ringSegments(parts);
  const [pointed, setPointed] = useState<null | SummaryKind>(preview?.highlighted ?? null);
  const { diameter, thickness } = compact ? SIZES.compact : SIZES.card;
  const freed = parts.find(({ kind }) => kind === 'can-be-freed');
  const free = parts.find(({ kind }) => kind === 'free');
  const scanned = phase === 'finished' || phase === 'partial';
  const name = { kind: 'text', value: volume.name } as const;

  return (
    <section
      aria-label={compact ? volume.name : t('ui.disk-card.label')}
      className={styles.card}
      data-compact={compact ? '' : undefined}
    >
      <div className={styles.ring} style={{ '--ring-pad': `${thickness / 2}px` } as CSSProperties}>
        <StorageRing
          accessibleName={t('ui.disk-card.ring', { name, parts: { kind: 'text', value: partsText(parts) } })}
          highlighted={pointed}
          label={volume.name}
          look={LOOK}
          onHighlight={(key) => {
            setPointed(parts.find(({ kind }) => kind === key)?.kind ?? null);
          }}
          segments={segments}
          size={diameter - thickness}
          state={RING_STATES[phase]}
          thickness={thickness}
          total={parts.reduce((sum, part) => sum + part.sizes.allocated, 0)}
        >
          <div className={styles.centre}>
            {phase === 'idle' && !compact ? (
              <Button onClick={onScan} variant="primary">
                {t('ui.scan-hero.scan')}
              </Button>
            ) : null}
            {scanned && freed !== undefined ? (
              <>
                <CentreSize sizes={freed.sizes} />
                <span className={styles.caption}>{t('ui.disk-card.can-be-freed')}</span>
              </>
            ) : null}
          </div>
        </StorageRing>
      </div>
      <div className={styles.side}>
        <div className={styles.head}>
          <span className={styles.name}>{volume.name}</span>
          {free === undefined ? null : (
            <FreeLine capacity={volume.capacity} free={honestSize(free.sizes).text ?? '—'} />
          )}
        </div>
        {phase === 'failed' ? (
          <ErrorState
            action={{ label: t('ui.scan-hero.try-again'), onClick: onScan }}
            text={failure === null ? '' : tText(failure)}
            title={t('ui.scan-hero.failed')}
          />
        ) : (
          <>
            <DiskLegend
              highlighted={pointed}
              initiallyOpen={preview?.open ?? null}
              onHighlight={setPointed}
              parts={parts}
              purgeable={volume.purgeable}
              scanned={scanned}
            />
            {volume.diskChanged ? (
              <p className={styles.note}>{tText({ args: {}, key: 'core.storage.changed' })}</p>
            ) : null}
          </>
        )}
      </div>
    </section>
  );
};
