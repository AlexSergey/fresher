import { type JSX, useId, useState } from 'react';

import type { Tone } from '../../design/data/segment';
import type { VolumeMap } from '../../ipc/bindings.generated';

import { UsageBar } from '../../design/data/usage-bar/usage-bar';
import { Icon } from '../../design/icons/icon';
import { ChevronRight } from '../../design/icons/icons';
import { Surface } from '../../design/layout/surface/surface';
import { t, tText } from '../../i18n';
import { PartLine } from './part-line';
import { segmentsOf } from './parts';
import styles from './storage-map.module.css';

export type StorageMapProps = {
  // A second volume: only its name shows until it is opened.
  readonly collapsible?: boolean;
  // The tone of Explained: the highest danger level of its categories, or a neutral tone when they are not known.
  readonly explained: Tone;
  // During a scan the parts' sizes count toward their values.
  readonly scanning: boolean;
  readonly volume: VolumeMap;
};

// Where one volume's space went: its name and kind, the bar of its parts with the legend, and a line per part with
// space. Nothing here offers a cleanup; the parts are drawn over their own sum, so a disk that changed during the scan
// still fills the bar.
export const StorageMap = ({ collapsible = false, explained, scanning, volume }: StorageMapProps): JSX.Element => {
  const [open, setOpen] = useState(!collapsible);
  const bodyId = useId();
  const title = t('ui.storage-map.title', {
    kind: { kind: 'text', value: tText({ args: {}, key: `core.storage.kind.${volume.kind}` }) },
    name: { kind: 'text', value: volume.name },
  });
  const segments = segmentsOf(volume.parts, explained);
  const total = segments.reduce((sum, { value }) => sum + value, 0);
  const lines = volume.parts.filter(({ sizes }) => sizes.allocated > 0);

  return (
    <Surface elevation={1} padding="large" radius="lg">
      <section aria-label={title} className={styles.map}>
        <h2 className={styles.title}>
          {collapsible ? (
            <button
              aria-controls={bodyId}
              aria-expanded={open}
              className={styles.volume}
              onClick={() => {
                setOpen(!open);
              }}
              type="button"
            >
              <span className={styles.chevron}>
                <Icon icon={ChevronRight} />
              </span>
              {title}
            </button>
          ) : (
            title
          )}
        </h2>
        <div className={styles.body} hidden={!open} id={bodyId}>
          <UsageBar
            label={t('ui.storage-map.label', { name: { kind: 'text', value: volume.name } })}
            legend
            segments={segments}
            total={total}
          />
          {volume.diskChanged ? (
            <p className={styles.note}>{tText({ args: {}, key: 'core.storage.changed' })}</p>
          ) : null}
          <ul
            aria-label={t('ui.storage-map.parts', { name: { kind: 'text', value: volume.name } })}
            className={styles.lines}
          >
            {lines.map((part, index) => (
              <PartLine
                key={`${part.kind.kind}-${String(index)}`}
                part={part}
                purgeable={volume.purgeable}
                scanning={scanning}
              />
            ))}
          </ul>
        </div>
      </section>
    </Surface>
  );
};
