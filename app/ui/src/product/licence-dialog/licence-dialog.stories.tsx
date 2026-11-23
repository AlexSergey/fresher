import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { LicenceDialog } from './licence-dialog';
import styles from './licence-dialog.stories.module.css';

const ignore = (): void => undefined;

const Open = (): JSX.Element => (
  <div className={styles.stage}>
    <LicenceDialog onClose={ignore} open />
  </div>
);

export const stories = storiesOf('LicenceDialog', {
  open: { render: Open, width: 1052 },
});
