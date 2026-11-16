import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Surface, type SurfaceProps } from './surface';
import styles from './surface.stories.module.css';

const card = (elevation: SurfaceProps['elevation']): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <Surface elevation={elevation}>
        <p className={styles.title}>Xcode caches</p>
        <p className={styles.text}>Build products and indexes Xcode makes again on the next build.</p>
      </Surface>
    );
  };

export const stories = storiesOf('Surface', {
  'elevation-1': { render: card(1), width: 360 },
  'elevation-2': { render: card(2), width: 360 },
  'elevation-3': { render: card(3), width: 360 },
});
