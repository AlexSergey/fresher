import type { JSX } from 'react';

import type { CategoryTotal, DangerLevel } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { CategoryCard } from '../category-card/category-card';
import { DEMO_TOTALS, withTexts } from '../scan-hero/story-data';
import { DangerZone, type DangerZoneProps } from './danger-zone';
import { ZONE_ORDER } from './zones';

const WIDTH = 760;

const [CACHES, DOWNLOADS, IMAGES, VOLUMES] = DEMO_TOTALS as readonly [
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
];

const TOTAL_OF: Readonly<Record<DangerLevel, CategoryTotal>> = {
  high: VOLUMES,
  low: DOWNLOADS,
  medium: IMAGES,
  safe: CACHES,
};

const noop = (): void => undefined;

const MENU = [
  {
    description: 'Hides it from the results and never removes it. You can restore it in Settings.',
    key: 'ignore-category',
    label: 'Ignore this category',
    onSelect: noop,
  },
];

// A zone with one card in it.
const zone = (level: DangerLevel, props: Partial<DangerZoneProps> = {}): (() => JSX.Element) =>
  withTexts(() => (
    <DangerZone level={level} selectAll={{ checked: false, onChange: noop }} {...props}>
      <CategoryCard menu={MENU} onDetails={noop} onToggle={noop} selected={false} share={0.4} total={TOTAL_OF[level]} />
    </DangerZone>
  ));

export const stories = storiesOf('DangerZone', {
  // One per level, safest first.
  ...Object.fromEntries(ZONE_ORDER.map((level) => [level, { render: zone(level), width: WIDTH }])),
  'select-all-mixed': { render: zone('safe', { selectAll: { checked: 'mixed', onChange: noop } }), width: WIDTH },
  'without-select-all': { render: zone('medium', { selectAll: undefined }), width: WIDTH },
});
