import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { AmbientLight, type AmbientLightProps } from './ambient-light';
import styles from './ambient-light.stories.module.css';

// The light behind a hero's number, as on the overview.
const hero = (props: AmbientLightProps): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={styles.stage}>
        <AmbientLight {...props} />
        <p className={styles.number}>42.8 GB</p>
      </div>
    );
  };

export const stories = storiesOf('AmbientLight', {
  // Motion off stands the loop at its first frame.
  'breathing-frozen': { render: hero({ breathing: true }), width: 720 },
  rest: { render: hero({}), width: 720 },
  'safe-tone': { render: hero({ tone: 'safe' }), width: 720 },
});
