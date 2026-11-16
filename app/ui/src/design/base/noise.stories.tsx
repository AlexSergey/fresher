import { type JSX, useLayoutEffect } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { AmbientLight } from '../effects/ambient-light/ambient-light';
import { Surface } from '../layout/surface/surface';
import styles from './noise.stories.module.css';

// The window's surfaces under the noise layer (effects.css): the sidebar's gradient, the main pane, a light and a
// card, where flat gradients show their bands without it.
const Surfaces = (): JSX.Element => (
  <div className={styles.window}>
    <div className={styles.sidebar} />
    <div className={styles.main}>
      <AmbientLight />
      <Surface elevation={1} radius="xl">
        <p className={styles.number}>42.8 GB</p>
      </Surface>
    </div>
  </div>
);

// The same window with the noise layer turned off for the page, to compare the two.
const WithoutNoise = (): JSX.Element => {
  useLayoutEffect(() => {
    document.documentElement.dataset.noise = 'off';

    return (): void => {
      delete document.documentElement.dataset.noise;
    };
  }, []);

  return <Surfaces />;
};

export const stories = storiesOf('Noise', {
  'window-with-noise': { render: Surfaces, width: 720 },
  'window-without-noise': { render: WithoutNoise, width: 720 },
});
