import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { CleanupProgress, type CleanupProgressProps } from './cleanup-progress';

const ignore = (): void => undefined;

const progress = (props: Partial<CleanupProgressProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <CleanupProgress
        done={0}
        method="trash"
        now="Example: editor-cache"
        onStop={ignore}
        stopping={false}
        total={128}
        {...props}
      />
    );
  };

export const stories = storiesOf('CleanupProgress', {
  deleting: {
    render: progress({ done: 40, method: 'permanent', now: 'Example: installer.dmg' }),
    surface: 'raised',
    width: 640,
  },
  early: { render: progress({ done: 11 }), surface: 'raised', width: 640 },
  late: { render: progress({ done: 120, now: 'Example: browser-cache' }), surface: 'raised', width: 640 },
  stopping: { render: progress({ done: 64, stopping: true }), surface: 'raised', width: 640 },
});
