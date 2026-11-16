import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Divider } from './divider';
import styles from './divider.stories.module.css';

const Horizontal = (): JSX.Element => (
  <div className={styles.column}>
    <span>Caches</span>
    <Divider />
    <span>Logs</span>
  </div>
);

const Vertical = (): JSX.Element => (
  <div className={styles.row}>
    <span>Caches</span>
    <Divider orientation="vertical" />
    <span>Logs</span>
  </div>
);

export const stories = storiesOf('Divider', {
  horizontal: { render: Horizontal, width: 240 },
  vertical: Vertical,
});
