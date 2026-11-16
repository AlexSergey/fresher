import type { JSX } from 'react';

import { useMotion } from '../../motion';
import styles from './ambient-light.module.css';

export type AmbientLightProps = {
  // Breathes slowly: only while work runs (a scan), never at rest.
  readonly breathing?: boolean;
  // `safe` is the cleaned moment's brief green.
  readonly tone?: AmbientTone;
};

type AmbientTone = 'iris' | 'safe';

const TONES: readonly AmbientTone[] = ['iris', 'safe'];

// A soft light behind a screen's hero, filling its positioned parent: a wide radial glow from the top centre and a
// fainter one inside it. Both tones are always drawn and crossfade by opacity. It is still under reduced motion and
// motion off, and off under increased contrast.
export const AmbientLight = ({ breathing = false, tone = 'iris' }: AmbientLightProps): JSX.Element => {
  const motion = useMotion();

  return (
    <div
      aria-hidden="true"
      className={styles.light}
      data-ambient-light
      data-breathing={(breathing && motion === 'full') || undefined}
    >
      {TONES.map((each) => (
        <div className={styles.glow} data-shown={each === tone || undefined} data-tone={each} key={each} />
      ))}
    </div>
  );
};
