import type { Segment } from '../segment';

import { stories as storiesOf } from '../../../gallery/story';
import { STORY_SEGMENTS, STORY_TOTAL } from '../chart-stories';
import { UsageBar } from './usage-bar';

const EIGHT = STORY_SEGMENTS.slice(0, 8);
const EIGHT_TOTAL = EIGHT.reduce((sum, { value }) => sum + value, 0);

const VOLUME: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', size: '38 GB', tone: 'safe', value: 38 },
  { key: 'docker', label: 'Docker', size: '22 GB', tone: 'medium', value: 22 },
  { key: 'your-folders', label: 'Your folders', size: '120 GB', tone: 'your-folders', value: 120 },
  { key: 'mac-os', label: 'macOS and swap', size: '60 GB', tone: 'mac-os', value: 60 },
  { key: 'unknown', label: 'Not scanned yet', size: '40 GB', tone: 'unknown', value: 40 },
  { key: 'free', label: 'Free', size: '220 GB', tone: 'free', value: 220 },
];

export const stories = storiesOf('UsageBar', {
  '1-segment': { render: () => <UsageBar label="Space found" segments={EIGHT.slice(0, 1)} total={1200} />, width: 480 },
  '8-segments': { render: () => <UsageBar label="Space found" segments={EIGHT} total={EIGHT_TOTAL} />, width: 480 },
  empty: { render: () => <UsageBar label="Space found" segments={[]} total={0} />, width: 480 },
  'whole-volume': {
    render: () => <UsageBar label="Macintosh HD" legend segments={VOLUME} total={500} />,
    width: 480,
  },
  'with-legend': {
    render: () => <UsageBar label="Space found" legend segments={EIGHT} total={STORY_TOTAL} />,
    width: 480,
  },
});
