import type { JSX } from 'react';

import type { Sizes } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { ScanningCard, type ScanningCardProps } from './scanning-card';

const WIDTH = 830;

const found = (allocated: number): Sizes => ({
  allocated,
  completeness: 'complete',
  logical: allocated,
  reclaimable: allocated,
});

const card =
  (props: Partial<ScanningCardProps>): (() => JSX.Element) =>
  () => (
    <ScanningCard
      found={found(1_900_000_000)}
      module="Developer caches"
      percent={62}
      volumeName="Macintosh HD"
      {...props}
    />
  );

export const stories = storiesOf('ScanningCard', {
  'no-module-known': { render: card({ module: null }), width: WIDTH },
  'part-62': { render: card({}), width: WIDTH },
  start: { render: card({ found: found(0), module: null, percent: 0 }), width: WIDTH },
});
