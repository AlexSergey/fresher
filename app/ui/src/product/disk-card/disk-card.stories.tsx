import type { JSX } from 'react';

import type { VolumeMap } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { DiskCard, type DiskCardProps } from './disk-card';
import { DISK_VOLUME, DISK_VOLUME_CHANGED, DISK_VOLUME_NOTHING, DISK_VOLUME_TINY, withTexts } from './story-data';

const WIDTH = 820;

const card = (props: Partial<DiskCardProps>, volume: VolumeMap = DISK_VOLUME): (() => JSX.Element) =>
  withTexts(() => <DiskCard failure={null} onScan={() => undefined} phase="finished" volume={volume} {...props} />);

export const stories = storiesOf('DiskCard', {
  compact: { render: card({ compact: true }), width: WIDTH },
  'disk-changed': { render: card({}, DISK_VOLUME_CHANGED), width: WIDTH },
  failed: {
    render: card({ failure: { args: {}, key: 'demo.error.failed' }, phase: 'failed' }),
    width: WIDTH,
  },
  finished: { render: card({}), width: WIDTH },
  'free-row-open': { render: card({ preview: { open: 'free' } }), width: WIDTH },
  'nothing-to-free': { render: card({}, DISK_VOLUME_NOTHING), width: WIDTH },
  'part-too-small-to-see': { render: card({}, DISK_VOLUME_TINY), width: WIDTH },
  'row-highlighted': { render: card({ preview: { highlighted: 'mac-os' } }), width: WIDTH },
  'scan-not-yet-run': { render: card({ phase: 'idle' }), width: WIDTH },
});
