import type { JSX } from 'react';

import { Tile } from '../../design/layout/tile/tile';
import styles from './placeholder-tile.module.css';

// The tile of what is still being found: the shapes of an icon, two lines and a size, faded, hidden from assistive
// technology.
export const PlaceholderTile = (): JSX.Element => (
  <li aria-hidden="true" className={styles.cell} data-placeholder>
    <Tile>
      <div className={styles.top}>
        <span className={styles.icon} />
        <div className={styles.lines}>
          <span className={styles.name} />
          <span className={styles.line} />
        </div>
      </div>
      <span className={styles.size} />
    </Tile>
  </li>
);
