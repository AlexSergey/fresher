import type { JSX } from 'react';

import { Announcer } from '../../design/feedback/announcer/announcer';
import { stories as storiesOf } from '../../gallery/story';
import { ScanHero, type ScanHeroProps } from './scan-hero';
import { DEMO_FINISHED, DEMO_PARTIAL, DEMO_RUNNING, NOTHING_FOUND, TWELVE_CATEGORIES, withTexts } from './story-data';

const WIDTH = 820;

const hero = (props: Partial<ScanHeroProps>): (() => JSX.Element) =>
  withTexts(() => (
    <Announcer>
      <ScanHero
        failure={null}
        onScan={() => undefined}
        onStop={() => undefined}
        overview={null}
        phase="idle"
        progress={null}
        {...props}
      />
    </Announcer>
  ));

export const stories = storiesOf('ScanHero', {
  failed: {
    render: hero({
      failure: { args: {}, key: 'demo.error.failed' },
      phase: 'failed',
    }),
    width: WIDTH,
  },
  'finished-12-categories': {
    render: hero({ overview: TWELVE_CATEGORIES.overview, phase: 'finished' }),
    width: WIDTH,
  },
  'finished-demo': { render: hero({ overview: DEMO_FINISHED, phase: 'finished' }), width: WIDTH },
  idle: { render: hero({}), width: WIDTH },
  'nothing-found': { render: hero({ overview: NOTHING_FOUND, phase: 'finished' }), width: WIDTH },
  partial: { render: hero({ overview: DEMO_PARTIAL, phase: 'partial' }), width: WIDTH },
  scanning: {
    render: hero({
      overview: DEMO_RUNNING,
      phase: 'scanning',
      progress: {
        entries: 48_213,
        modules: [
          { group: 'debug', module: 'demo', state: { kind: 'running' } },
          { group: 'development', module: 'xcode', state: { kind: 'completed' } },
        ],
      },
    }),
    width: WIDTH,
  },
});
