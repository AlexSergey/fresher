import type { JSX } from 'react';

import type { VolumeMap } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { StorageMap } from './storage-map';
import {
  COMPLETE,
  EXTERNAL,
  NO_SCAN,
  NOT_MEASURED,
  OUTSIDE_SCAN,
  SCANNING,
  WITHOUT_VOLUME_FACTS,
  withStorageTexts,
} from './story-data';

const WIDTH = 720;

const map = (volume: VolumeMap, scanning = false): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: withStorageTexts(() => <StorageMap explained="low" scanning={scanning} volume={volume} />),
  width: WIDTH,
});

export const stories = storiesOf('StorageMap', {
  complete: map(COMPLETE),
  'no-scan': map(NO_SCAN),
  'not-measured': map(NOT_MEASURED),
  'outside-scan': map(OUTSIDE_SCAN),
  scanning: map(SCANNING, true),
  'two-volumes': {
    render: withStorageTexts(() => (
      <div style={{ display: 'grid', gap: 16 }}>
        <StorageMap explained="medium" scanning={false} volume={COMPLETE} />
        <StorageMap collapsible explained="safe" scanning={false} volume={EXTERNAL} />
      </div>
    )),
    width: WIDTH,
  },
  'without-volume-facts': map(WITHOUT_VOLUME_FACTS),
});
