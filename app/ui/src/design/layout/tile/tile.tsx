import type { JSX, ReactNode } from 'react';

import type { Level } from '../../data/danger-badge/danger-badge';

import styles from './tile.module.css';

export type TileProps = {
  readonly as?: 'article' | 'div' | 'li' | 'section';
  readonly children?: ReactNode;
  // The id of the heading that names the tile (an `article`'s or a `section`'s).
  readonly labelledBy?: string;
  // 'none' for a tile whose header and list carry their own padding (Review's).
  readonly padding?: 'none' | 'normal';
  // 'medium' is the warm surface of the "Look before removing" tiles.
  readonly surface?: 'medium' | 'plain';
  // The danger level whose border the tile takes; without one, the neutral border.
  readonly tone?: Level | undefined;
};

// The card every category is drawn on: a gradient fill, a hairline in its level's colour and a top highlight.
export const Tile = ({
  as: Tag = 'div',
  children,
  labelledBy,
  padding = 'normal',
  surface = 'plain',
  tone,
}: TileProps): JSX.Element => (
  <Tag
    aria-labelledby={labelledBy}
    className={styles.tile}
    data-level={tone}
    data-padding={padding}
    data-surface={surface}
  >
    {children}
  </Tag>
);
