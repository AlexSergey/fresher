import type { JSX } from 'react';

import type { VolumeMap } from '../../ipc/bindings.generated';

import { Announcer } from '../../design/feedback/announcer/announcer';
import { stories as storiesOf } from '../../gallery/story';
import { DEMO_MAP_COMPLETE, DEMO_MAP_NO_SCAN, DEMO_MAP_SCANNING } from '../../test/storage-map';
import { levelShares } from './ring-parts';
import { ScanHero, type ScanHeroProps } from './scan-hero';
import {
  DEMO_FINISHED,
  DEMO_PARTIAL,
  DEMO_RUNNING,
  HERO_LEVELS,
  HERO_OVERVIEW,
  HERO_VOLUME,
  HERO_VOLUME_CHANGED,
  NO_LEVELS,
  NOTHING_FOUND,
  TWELVE_CATEGORIES,
  withTexts,
} from './story-data';

const WIDTH = 820;

const hero = (props: Partial<ScanHeroProps>): (() => JSX.Element) =>
  withTexts(() => (
    <Announcer>
      <ScanHero
        failure={null}
        levels={NO_LEVELS}
        onScan={() => undefined}
        onStop={() => undefined}
        overview={null}
        phase="idle"
        progress={null}
        volume={null}
        {...props}
      />
    </Announcer>
  ));

const homeOf = (map: typeof DEMO_MAP_COMPLETE): null | VolumeMap => map.volumes[0] ?? null;

const finished = { overview: DEMO_FINISHED, phase: 'finished' } as const;
const sample = { levels: HERO_LEVELS, overview: HERO_OVERVIEW, phase: 'finished', volume: HERO_VOLUME } as const;

export const stories = storiesOf('ScanHero', {
  failed: {
    render: hero({
      failure: { args: {}, key: 'demo.error.failed' },
      phase: 'failed',
      volume: homeOf(DEMO_MAP_NO_SCAN),
    }),
    width: WIDTH,
  },
  'finished-12-categories': {
    render: hero({
      ...sample,
      levels: levelShares(null, TWELVE_CATEGORIES.overview.totals),
      overview: TWELVE_CATEGORIES.overview,
      phase: 'finished',
    }),
    width: WIDTH,
  },
  'finished-demo': {
    render: hero({
      ...finished,
      levels: levelShares(null, DEMO_FINISHED.totals),
      volume: homeOf(DEMO_MAP_COMPLETE),
    }),
    width: WIDTH,
  },
  'finished-disk-changed': {
    render: hero({ ...sample, volume: HERO_VOLUME_CHANGED }),
    width: WIDTH,
  },
  highlighted: {
    render: hero({ ...sample, preview: { highlighted: 'level-low' } }),
    width: WIDTH,
  },
  idle: { render: hero({ volume: homeOf(DEMO_MAP_NO_SCAN) }), width: WIDTH },
  'legend-open': {
    render: hero({ ...sample, preview: { open: 'remainder-2' } }),
    width: WIDTH,
  },
  'nothing-found': {
    render: hero({ overview: NOTHING_FOUND, phase: 'finished', volume: homeOf(DEMO_MAP_COMPLETE) }),
    width: WIDTH,
  },
  partial: {
    render: hero({
      levels: levelShares(null, DEMO_PARTIAL.totals),
      overview: DEMO_PARTIAL,
      phase: 'partial',
      volume: homeOf(DEMO_MAP_COMPLETE),
    }),
    width: WIDTH,
  },
  scanning: {
    render: hero({
      levels: levelShares(null, DEMO_RUNNING.totals),
      overview: DEMO_RUNNING,
      phase: 'scanning',
      progress: {
        entries: 48_213,
        modules: [
          { group: 'debug', module: 'demo', state: { kind: 'running' } },
          { group: 'development', module: 'xcode', state: { kind: 'completed' } },
        ],
      },
      volume: homeOf(DEMO_MAP_SCANNING),
    }),
    width: WIDTH,
  },
});
