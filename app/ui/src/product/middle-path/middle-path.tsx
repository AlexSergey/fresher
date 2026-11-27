import type { JSX } from 'react';

import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import styles from './middle-path.module.css';

export type MiddlePathProps = {
  // A path for display (`~/Code/app`).
  readonly path: string;
};

// The last two segments stay whole, so the user sees which folder it is and where it is.
const TAIL_SEGMENTS = 2;

// A path that is cut in the middle when the room is short: the first folder names shrink to an ellipsis, the last two
// segments never do. A path of two segments or fewer is one piece that ends in an ellipsis. The full path is the
// title, and the one text assistive technology reads.
export const MiddlePath = ({ path }: MiddlePathProps): JSX.Element => {
  const segments = path.split('/');
  const cut = segments.length - TAIL_SEGMENTS;
  const head = cut > 0 ? segments.slice(0, cut).join('/') : path;
  const tail = cut > 0 ? `/${segments.slice(cut).join('/')}` : '';

  return (
    <span className={styles.path} title={path}>
      <VisuallyHidden>{path}</VisuallyHidden>
      <span aria-hidden="true" className={styles.head}>
        {head}
      </span>
      {tail === '' ? null : (
        <span aria-hidden="true" className={styles.tail}>
          {tail}
        </span>
      )}
    </span>
  );
};
