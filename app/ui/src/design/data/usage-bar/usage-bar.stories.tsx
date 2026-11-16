import { stories as storiesOf } from '../../../gallery/story';
import { STORY_SEGMENTS, STORY_TOTAL } from '../chart-stories';
import { UsageBar } from './usage-bar';

const EIGHT = STORY_SEGMENTS.slice(0, 8);
const EIGHT_TOTAL = EIGHT.reduce((sum, { value }) => sum + value, 0);

export const stories = storiesOf('UsageBar', {
  '1-segment': { render: () => <UsageBar label="Space found" segments={EIGHT.slice(0, 1)} total={1200} />, width: 480 },
  '8-segments': { render: () => <UsageBar label="Space found" segments={EIGHT} total={EIGHT_TOTAL} />, width: 480 },
  empty: { render: () => <UsageBar label="Space found" segments={[]} total={0} />, width: 480 },
  'with-legend': {
    render: () => <UsageBar label="Space found" legend segments={EIGHT} total={STORY_TOTAL} />,
    width: 480,
  },
});
