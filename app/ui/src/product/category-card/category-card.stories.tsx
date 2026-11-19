import { type JSX, useLayoutEffect, useRef } from 'react';

import type { CategoryTotal } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { DEMO_TOTALS, withTexts } from '../scan-hero/story-data';
import { CategoryCard, type CategoryCardProps } from './category-card';

const WIDTH = 360;

const [CACHES, DOWNLOADS, IMAGES, VOLUMES] = DEMO_TOTALS as readonly [
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
  CategoryTotal,
];

const card = (total: CategoryTotal, overrides: Partial<CategoryCardProps> = {}): (() => JSX.Element) =>
  withTexts(() => (
    <CategoryCard
      onOpen={() => undefined}
      onToggle={() => undefined}
      selected={false}
      share={0.6}
      total={total}
      {...overrides}
    />
  ));

// The card as a resting pointer would show it: lifted, its border brighter, the spotlight at its centre.
const Hovered = (): JSX.Element => {
  const frameRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const root = frameRef.current?.querySelector<HTMLElement>('[data-card]');
    const surface = root?.firstElementChild;

    if (root !== null && root !== undefined && surface instanceof HTMLElement) {
      root.dataset.hover = '';
      surface.dataset.hover = '';
      surface.style.setProperty('--spot-x', `${String(surface.offsetWidth / 2)}px`);
      surface.style.setProperty('--spot-y', `${String(surface.offsetHeight / 2)}px`);
    }
  }, []);

  return <div ref={frameRef}>{card(CACHES)()}</div>;
};

export const stories = storiesOf('CategoryCard', {
  high: { render: card(VOLUMES, { share: 0.05 }), width: WIDTH },
  hover: { render: Hovered, width: WIDTH },
  limited: {
    render: card(
      { ...CACHES, sizes: { ...CACHES.sizes, completeness: 'partial' } },
      { access: { kind: 'limited', text: 'Some of it could not be seen: Full Disk Access.' } },
    ),
    width: WIDTH,
  },
  low: { render: card(DOWNLOADS, { share: 0.25 }), width: WIDTH },
  medium: { render: card(IMAGES, { share: 0.14 }), width: WIDTH },
  mixed: { render: card(CACHES, { selected: 'mixed' }), width: WIDTH },
  'partial-size': {
    render: card({ ...CACHES, sizes: { ...CACHES.sizes, completeness: 'partial' } }),
    width: WIDTH,
  },
  safe: { render: card(CACHES), width: WIDTH },
  selected: { render: card(CACHES, { selected: true }), width: WIDTH },
  unavailable: {
    render: card(
      { ...IMAGES, count: 0, sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 } },
      { access: { kind: 'unavailable', text: 'Docker is installed but not running. Start it, then scan again.' } },
    ),
    width: WIDTH,
  },
});
