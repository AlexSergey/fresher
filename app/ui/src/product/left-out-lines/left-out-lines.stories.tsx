import { stories as storiesOf } from '../../gallery/story';
import { DEMO_LEFT_OUT } from '../../test/demo-scan';
import { forNowCount, keptCount, leftOutEntry } from '../../test/left-out';
import { withTexts } from '../../test/story-data';
import { LeftOutLines } from './left-out-lines';

const MB = 1_000_000;

const sized = (bytes: number): typeof DEMO_LEFT_OUT.sizes => ({
  allocated: bytes,
  completeness: 'complete',
  logical: bytes,
  reclaimable: bytes,
});

// Twelve caches of apps that are open: the two largest listed, the rest only counted.
const MANY = forNowCount(
  'demo.caches',
  'demo.left-out.open',
  sized(1_240 * MB),
  [
    leftOutEntry(
      { args: { name: { kind: 'text', value: 'com.example.editor' } }, key: 'demo.finding.title' },
      '/Users/someone/Library/Caches/com.example.editor',
      sized(820 * MB),
    ),
    leftOutEntry(
      { args: { name: { kind: 'text', value: 'com.example.music' } }, key: 'demo.finding.title' },
      '/Users/someone/Library/Caches/com.example.music',
      sized(310 * MB),
    ),
  ],
  12,
);

// The version in use, kept for good: one quiet line, no item.
const KEPT = keptCount('demo.caches', 'demo.left-out.open', 3, sized(1_400 * MB));

export const stories = storiesOf('LeftOutLines', {
  'details-bars': {
    render: withTexts(() => <LeftOutLines bar counts={[MANY, KEPT]} home="/Users/someone" />),
    width: 560,
  },
  'one-line': { render: withTexts(() => <LeftOutLines counts={[DEMO_LEFT_OUT]} />), width: 560 },
  'with-more': {
    render: withTexts(() => <LeftOutLines counts={[MANY]} home="/Users/someone" />),
    width: 560,
  },
});
