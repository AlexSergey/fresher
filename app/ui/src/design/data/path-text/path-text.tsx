import { type JSX, use, useLayoutEffect, useRef, useState } from 'react';

import { Tooltip } from '../../primitives/tooltip/tooltip';
import { VisuallyHidden } from '../../primitives/visually-hidden/visually-hidden';
import { PathFormContext } from './path-form';
import styles from './path-text.module.css';
import { truncateMiddle } from './truncate';

export type PathTextProps = {
  // What is shown in place of the path (`~/…`); the tooltip and the accessible text keep the full path.
  readonly display?: string;
  readonly path: string;
};

// A run of mono cells measured once per resize: every character of the mono font is one cell wide.
const PROBE = '0000000000';

// A path in mono, cut in the middle to fit its width, so its start and its file name stay; the full path is in its
// tooltip and in its accessible text, each written in the path form around it. Measured again whenever its width
// changes.
export const PathText = ({ display, path }: PathTextProps): JSX.Element => {
  const form = use(PathFormContext);
  const full = form(path);
  const shown = form(display ?? path);
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
      <Tooltip content={full} repeatsName>
        {() => <span aria-hidden="true">{fit === null ? shown : truncateMiddle(shown, fit)}</span>}
      </Tooltip>
      <VisuallyHidden>{full}</VisuallyHidden>
    </span>
  );
};
