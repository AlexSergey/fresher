import { stories as storiesOf } from '../../gallery/story';
import { DEMO_LEFT_OUT } from '../../test/demo-scan';
import { withTexts } from '../scan-hero/story-data';
import { LeftOutLines } from './left-out-lines';

export const stories = storiesOf('LeftOutLines', {
  'one-line': { render: withTexts(() => <LeftOutLines counts={[DEMO_LEFT_OUT]} />), width: 560 },
  'two-lines': {
    render: withTexts(() => (
      <LeftOutLines
        counts={[DEMO_LEFT_OUT, { ...DEMO_LEFT_OUT, items: 3, reason: { args: {}, key: 'demo.error.failed' } }]}
      />
    )),
    width: 560,
  },
});
