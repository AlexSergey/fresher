import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { RecoveryHelp } from './recovery-help';
import styles from './recovery-help.stories.module.css';

const Closed = (): JSX.Element => <RecoveryHelp />;

const Open = (): JSX.Element => (
  <div className={styles.stage}>
    <RecoveryHelp defaultOpen />
  </div>
);

export const stories = storiesOf('RecoveryHelp', {
  closed: { render: Closed, width: 320 },
  open: { render: Open, width: 1052 },
});
