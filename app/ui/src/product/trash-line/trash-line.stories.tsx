import type { JSX } from 'react';

import type { Sizes, TrashLine } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { TrashLines } from './trash-line';

const ignore = (): void => undefined;

const sizes = (allocated: number): Sizes => ({
  allocated,
  completeness: 'complete',
  logical: allocated,
  reclaimable: allocated,
});

const line = (props: Partial<TrashLine> = {}): TrashLine => ({
  fromCleanups: sizes(1_200_000_000),
  goneBy: null,
  index: 0,
  volume: 'Macintosh HD',
  waiting: sizes(5_000_000_000),
  ...props,
});

const lines = (value: readonly TrashLine[]): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <TrashLines lines={value} onOpen={ignore} />;
  };

export const stories = storiesOf('TrashLines', {
  'all-ours': { render: lines([line({ waiting: sizes(1_200_000_000) })]), surface: 'raised', width: 560 },
  'no-share': { render: lines([line({ fromCleanups: sizes(0) })]), surface: 'raised', width: 560 },
  'not-measured': { render: lines([line({ waiting: null })]), surface: 'raised', width: 560 },
  share: { render: lines([line()]), surface: 'raised', width: 560 },
  'two-volumes': {
    render: lines([line(), line({ fromCleanups: sizes(0), index: 1, volume: 'Backup', waiting: sizes(300_000_000) })]),
    surface: 'raised',
    width: 560,
  },
  'with-date': { render: lines([line({ goneBy: 1_767_225_600 })]), surface: 'raised', width: 560 },
});
