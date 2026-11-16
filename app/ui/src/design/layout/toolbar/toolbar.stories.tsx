import { type JSX, useEffect, useRef } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { RotateCw } from '../../icons/icons';
import { IconButton } from '../../primitives/icon-button/icon-button';
import { ScrollArea } from '../scroll-area/scroll-area';
import { Toolbar } from './toolbar';
import styles from './toolbar.stories.module.css';

const LINES = Array.from({ length: 24 }, (_, index) => `Finding ${String(index + 1)}`);

// A toolbar over a scrolling list; `scrolledBy` pixels of it already scrolled under the toolbar.
const screen = (scrolledBy: number): (() => JSX.Element) =>
  function Story(): JSX.Element {
    const stageRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
      const area = stageRef.current?.firstElementChild;

      if (area instanceof HTMLElement) {
        area.scrollTop = scrolledBy;
      }
    }, []);

    return (
      <div className={styles.stage} ref={stageRef}>
        <ScrollArea label="Findings">
          <Toolbar actions={<IconButton icon={RotateCw} label="Scan again" />} title="Developer caches" />
          <ul className={styles.list}>
            {LINES.map((line) => (
              <li className={styles.line} key={line}>
                {line}
              </li>
            ))}
          </ul>
        </ScrollArea>
      </div>
    );
  };

export const stories = storiesOf('Toolbar', {
  rest: { render: screen(0), width: 560 },
  scrolled: { render: screen(120), width: 560 },
});
