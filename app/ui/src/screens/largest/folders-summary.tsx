import type { JSX } from 'react';

import type { Segment, Tone } from '../../design/data/segment';
import type { IconShape } from '../../design/icons/icons';
import type { FoldersPage } from '../../ipc/bindings.generated';

import { UsageBar } from '../../design/data/usage-bar/usage-bar';
import { Icon } from '../../design/icons/icon';
import { Code, Folder, HardDrive } from '../../design/icons/icons';
import { SectionSeparator } from '../../design/layout/section-separator/section-separator';
import { Tile } from '../../design/layout/tile/tile';
import { t } from '../../i18n';
import { formatCount, sizeLabel } from '../../product/sizes';
import styles from './largest-screen.module.css';

// The largest folders' colours on the bar, in order; the rest of the list takes the last one.
const TONES: readonly Tone[] = ['files-and-apps', 'mac-os', 'system-space', 'swap-and-sleep', 'other-volume'];
const REST: Tone = 'your-folders';

// The largest folders drawn on the bar one by one; the others share one segment.
const NAMED = TONES.length;

// The listed folders on the home's bar: the five largest one by one, the rest of the list as one segment.
const segmentsOf = ({ rows, sizes, total }: FoldersPage): readonly Segment[] => {
  const named = rows.slice(0, NAMED);
  const segments: Segment[] = named.map(({ node }, at) => ({
    key: `${String(node.id.state)}:${String(node.id.index)}`,
    label: node.name,
    size: sizeLabel(node.sizes),
    tone: TONES[at] ?? REST,
    value: node.sizes.allocated,
  }));
  const others = total - named.length;

  if (others > 0) {
    const rest = sizes.allocated - named.reduce((sum, { node }) => sum + node.sizes.allocated, 0);
    segments.push({
      key: 'rest',
      label: t('ui.screen.largest.folders.legend-more', { count: { kind: 'count', value: others } }),
      size: sizeLabel({ ...sizes, allocated: rest }),
      tone: REST,
      value: rest,
    });
  }

  return segments;
};

// "50.4 GB of your home's 212 GB", the listed size bold; the dictionary's order of the words is kept. A line that does
// not hold the size as written is shown plain.
const Caption = ({ line, size }: { readonly line: string; readonly size: string }): JSX.Element => {
  const at = line.indexOf(size);

  return at < 0 ? (
    <p className={styles.caption}>{line}</p>
  ) : (
    <p className={styles.caption}>
      {line.slice(0, at)}
      <b>{size}</b>
      {line.slice(at + size.length)}
    </p>
  );
};

type Fact = { readonly icon: IconShape; readonly label: string; readonly tone: Tone; readonly value: string };

// The list's three facts, each with its icon in one of the bar's first colours.
const factsOf = (page: FoldersPage): readonly Fact[] => [
  {
    icon: Folder,
    label: t('ui.screen.largest.folders.tile.count'),
    tone: 'files-and-apps',
    value: formatCount(page.total),
  },
  { icon: HardDrive, label: t('ui.screen.largest.folders.tile.space'), tone: 'mac-os', value: sizeLabel(page.sizes) },
  {
    icon: Code,
    label: t('ui.screen.largest.folders.tile.projects'),
    tone: 'system-space',
    value: formatCount(page.projects),
  },
];

// Above the rows: the home's bar with the listed folders on it, its caption and legend, then "In this list" and the
// list's tiles. Every number is the core's.
export const FoldersSummary = ({ page }: { readonly page: FoldersPage }): JSX.Element => {
  const size = sizeLabel(page.sizes);
  const line = t('ui.screen.largest.folders.bar', {
    home: { kind: 'text', value: sizeLabel(page.home) },
    size: { kind: 'text', value: size },
  });

  return (
    <>
      <div className={styles.card}>
        <UsageBar
          caption={<Caption line={line} size={size} />}
          label={line}
          legend
          legendColumns={3}
          segments={segmentsOf(page)}
          total={page.home.allocated}
        />
      </div>
      <SectionSeparator label={t('ui.screen.largest.folders.separator')} />
      <ul className={styles.tiles}>
        {factsOf(page).map(({ icon, label, tone, value }) => (
          <Tile as="li" key={label}>
            <span className={styles.fact}>
              <span className={styles.factIcon} data-tone={tone}>
                <Icon icon={icon} size={20} />
              </span>
              <span className={styles.factLabel}>{label}</span>
              <span className={styles.factValue} data-numeric>
                {value}
              </span>
            </span>
          </Tile>
        ))}
      </ul>
    </>
  );
};
