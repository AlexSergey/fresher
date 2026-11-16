import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { DangerBadge, type Level } from './danger-badge';
import styles from './danger-badge.stories.module.css';

const LEVELS: Readonly<Record<Level, { readonly description: string; readonly label: string }>> = {
  high: { description: 'Work or settings you may not get back. Look before you remove.', label: 'High' },
  low: { description: 'Comes back on its own, but an app may start slower once.', label: 'Low' },
  medium: { description: 'Downloads or builds that take time to get again.', label: 'Medium' },
  safe: { description: 'Rebuilt by the app when it needs it. Nothing is lost.', label: 'Safe' },
};

// One level in its three sizes: small and medium in a line, large with its sentence.
const sizes = (level: Level): (() => JSX.Element) =>
  function Story(): JSX.Element {
    const { description, label } = LEVELS[level];

    return (
      <div className={styles.column}>
        <div className={styles.row}>
          <DangerBadge label={label} level={level} size="small" />
          <DangerBadge label={label} level={level} />
        </div>
        <DangerBadge description={description} label={label} level={level} size="large" />
      </div>
    );
  };

export const stories = storiesOf('DangerBadge', {
  high: { render: sizes('high'), width: 360 },
  low: { render: sizes('low'), width: 360 },
  medium: { render: sizes('medium'), width: 360 },
  safe: { render: sizes('safe'), width: 360 },
});
