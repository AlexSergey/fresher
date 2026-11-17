import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { withTexts } from '../scan-hero/story-data';
import { type ModuleProgress, ScanProgress } from './scan-progress';

const WIDTH = 480;

const progress = (entries: number, modules: readonly ModuleProgress[]): (() => JSX.Element) =>
  withTexts(() => <ScanProgress entries={entries} modules={modules} onStop={() => undefined} />);

export const stories = storiesOf('ScanProgress', {
  cancelled: {
    render: progress(31_877, [
      { group: 'debug', module: 'demo', state: { kind: 'completed' } },
      { group: 'development', module: 'xcode', state: { kind: 'cancelled' } },
      { group: 'development', module: 'node', state: { kind: 'cancelled' } },
    ]),
    width: WIDTH,
  },
  'module-failed': {
    render: progress(12_408, [
      { group: 'debug', module: 'demo', state: { kind: 'completed' } },
      {
        group: 'development',
        module: 'docker',
        state: { kind: 'failed', reason: { args: {}, key: 'docker.error.unreachable' } },
      },
      { group: 'development', module: 'xcode', state: { kind: 'running' } },
    ]),
    width: WIDTH,
  },
  running: {
    render: progress(48_213, [
      { group: 'debug', module: 'demo', state: { kind: 'completed' } },
      { group: 'development', module: 'xcode', state: { kind: 'running' } },
      { group: 'development', module: 'node', state: { kind: 'running' } },
      { group: 'development', module: 'logs', state: { kind: 'waiting' } },
    ]),
    width: WIDTH,
  },
  starting: {
    render: progress(0, [
      { group: 'debug', module: 'demo', state: { kind: 'waiting' } },
      { group: 'development', module: 'xcode', state: { kind: 'waiting' } },
      { group: 'development', module: 'node', state: { kind: 'waiting' } },
    ]),
    width: WIDTH,
  },
});
