import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Sheet } from './sheet';
import styles from './sheet.stories.module.css';

const noop = (): void => undefined;

const Open = (): JSX.Element => (
  <div className={styles.stage}>
    <p className={styles.behind}>Developer caches: 18.4 GB in 6 categories.</p>
    <Sheet
      closeLabel="Close"
      description="~/Library/Developer/Xcode/DerivedData"
      onClose={noop}
      open
      title="Xcode caches"
    >
      <p className={styles.text}>
        Build products and indexes Xcode makes for each project. Xcode makes them again on the next build, which takes
        longer the first time.
      </p>
    </Sheet>
  </div>
);

export const stories = storiesOf('Sheet', {
  open: { render: Open, width: 1052 },
});
