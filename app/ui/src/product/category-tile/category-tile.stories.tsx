import type { JSX } from 'react';

import type { CategoryTotal } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { DEMO_TOTALS, withTexts } from '../../test/story-data';
import { CategoryTile, type CategoryTileProps } from './category-tile';

const WIDTH = 360;

const [CACHES, DOWNLOADS, IMAGES, VOLUMES] = DEMO_TOTALS as readonly [
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
];

const MENU: CategoryTileProps['menu'] = [
  {
    key: 'ignore-category',
    label: 'Ignore all Example caches',
    onSelect: () => undefined,
  },
];

const tile = (total: CategoryTotal, overrides: Partial<CategoryTileProps> = {}): (() => JSX.Element) =>
  withTexts(() => (
    <CategoryTile
      icon="cache"
      menu={MENU}
      onDetails={() => undefined}
      onToggle={() => undefined}
      selected={false}
      total={total}
      {...overrides}
    />
  ));

export const stories = storiesOf('CategoryTile', {
  high: { render: tile(VOLUMES, { icon: 'archive', onToggle: undefined }), width: WIDTH },
  limited: {
    render: tile(
      { ...CACHES, sizes: { ...CACHES.sizes, completeness: 'partial' } },
      { access: { kind: 'limited', text: 'Some of it could not be seen: Full Disk Access.' } },
    ),
    width: WIDTH,
  },
  low: { render: tile(DOWNLOADS, { icon: 'download' }), width: WIDTH },
  medium: { render: tile(IMAGES, { icon: 'app', onToggle: undefined }), width: WIDTH },
  // The menu is drawn on <body>, under the tile: the story leaves it room inside the photographed frame.
  'menu-open': {
    render: withTexts(() => <div style={{ paddingBottom: 140 }}>{tile(CACHES, { menuOpen: true })()}</div>),
    width: WIDTH,
  },
  mixed: { render: tile(CACHES, { selected: 'mixed' }), width: WIDTH },
  rest: { render: tile(CACHES), width: WIDTH },
  scanning: { render: tile(CACHES, { scanning: true }), width: WIDTH },
  selected: { render: tile(CACHES, { selected: true }), width: WIDTH },
  unavailable: {
    render: tile(
      { ...IMAGES, count: 0, sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 } },
      { access: { kind: 'unavailable', text: 'Docker is installed but not running. Start it, then scan again.' } },
    ),
    width: WIDTH,
  },
});
