import { type JSX, useLayoutEffect, useRef, useState } from 'react';

import { Tooltip } from '../../primitives/tooltip/tooltip';
import { VisuallyHidden } from '../../primitives/visually-hidden/visually-hidden';
import styles from './path-text.module.css';
import { truncateMiddle } from './truncate';

export type PathTextProps = {
  readonly path: string;
};

// A run of mono cells measured once per resize: every character of the mono font is one cell wide.
const PROBE = '0000000000';

// A path in mono, cut in the middle to fit its width, so its start and its file name stay; the full path is in its
// tooltip and in its accessible text. Measured again whenever its width changes.
export const PathText = ({ path }: PathTextProps): JSX.Element => {
  const frameRef = useRef<HTMLSpanElement>(null);
  const probeRef = useRef<HTMLSpanElement>(null);
  // How many characters fit; `null` until measured, when the whole path shows.
  const [fit, setFit] = useState<null | number>(null);

  useLayoutEffect(() => {
    const frame = frameRef.current;
    const probe = probeRef.current;

    if (frame === null || probe === null) {
      return;
    }

    const measure = (): void => {
      const cell = probe.getBoundingClientRect().width / PROBE.length;
      const available = frame.clientWidth;

      setFit(cell > 0 && available > 0 ? Math.floor(available / cell) : null);
    };

    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(frame);

    return (): void => {
      observer.disconnect();
    };
  }, []);

  return (
    <span className={styles.frame} ref={frameRef}>
      <span aria-hidden="true" className={styles.probe} ref={probeRef}>
        {PROBE}
      </span>
      <Tooltip content={path} repeatsName>
        {() => <span aria-hidden="true">{fit === null ? path : truncateMiddle(path, fit)}</span>}
      </Tooltip>
      <VisuallyHidden>{path}</VisuallyHidden>
    </span>
  );
};
