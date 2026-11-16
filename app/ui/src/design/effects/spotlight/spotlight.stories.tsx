import { type JSX, useLayoutEffect } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Surface } from '../../layout/surface/surface';
import { useSpotlightProps } from './spotlight';
import styles from './spotlight.stories.module.css';

// A raised card with its spotlight shown at its centre, as a pointer resting there would place it.
const AtCentre = (): JSX.Element => {
  const { className, ref } = useSpotlightProps();

  useLayoutEffect(() => {
    const card = ref.current;

    if (card !== null) {
      card.style.setProperty('--spot-x', `${String(card.offsetWidth / 2)}px`);
      card.style.setProperty('--spot-y', `${String(card.offsetHeight / 2)}px`);
      card.dataset.hover = '';
    }
  }, [ref]);

  return (
    <Surface className={className} elevation={1} ref={ref}>
      <p className={styles.title}>Xcode caches</p>
      <p className={styles.text}>Build products and indexes Xcode makes again on the next build.</p>
    </Surface>
  );
};

export const stories = storiesOf('Spotlight', {
  'pointer-at-centre': { render: AtCentre, width: 400 },
});
