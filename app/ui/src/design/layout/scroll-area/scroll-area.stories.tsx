import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { ScrollArea } from './scroll-area';
import styles from './scroll-area.stories.module.css';

const paragraphs = (count: number): readonly string[] =>
  Array.from({ length: count }, (_, index) => `Paragraph ${String(index + 1)}: what the scan found in this folder.`);

const area = (count: number): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={styles.stage}>
        <ScrollArea label="Details">
          {paragraphs(count).map((text) => (
            <p className={styles.paragraph} key={text}>
              {text}
            </p>
          ))}
        </ScrollArea>
      </div>
    );
  };

export const stories = storiesOf('ScrollArea', {
  overflowing: { render: area(16), width: 360 },
  short: { render: area(2), width: 360 },
});
