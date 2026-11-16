import { type JSX, useEffect, useRef } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Checkbox } from '../../primitives/checkbox/checkbox';
import { ListRow } from '../list-row/list-row';
import { VirtualList } from './virtual-list';
import styles from './virtual-list.stories.module.css';

const ROW = 44;

const ignore = (): void => undefined;

const keyOf = (index: number): string => `cache-${String(index)}`;

const titleOf = (index: number): string => `Cache ${(index + 1).toLocaleString('en-US')}`;

// A size that looks real and is the same on every run.
const sizeOf = (index: number): string => `${(((index * 37) % 900) / 10 + 0.4).toFixed(1)} MB`;

const Rows = ({ count, scrollTo = 0 }: { readonly count: number; readonly scrollTo?: number }): JSX.Element => {
  const frameRef = useRef<HTMLDivElement>(null);

  // The list scrolls as a person would; it hears of it through its scroll event.
  useEffect(() => {
    const viewport = frameRef.current?.querySelector('[role="list"]');

    if (viewport !== null && viewport !== undefined) {
      viewport.scrollTop = scrollTo * ROW;
    }
  }, [scrollTo]);

  return (
    <div className={styles.frame} ref={frameRef}>
      <VirtualList
        count={count}
        getItem={titleOf}
        getKey={keyOf}
        label="Caches"
        renderRow={(index, title, row) => (
          <ListRow
            {...row}
            leading={
              // The row is the list's one tab stop, as a finding row is; Space on it ticks the checkbox.
              <Checkbox
                checked={index % 3 === 0}
                hideLabel
                label={`Select ${title ?? ''}`}
                onChange={ignore}
                tabIndex={-1}
              />
            }
            selected={index % 3 === 0}
            subtitle={`~/Library/Caches/com.example.app-${String(index)}`}
            title={title ?? ''}
            trailing={sizeOf(index)}
          />
        )}
        rowHeight={ROW}
      />
    </div>
  );
};

export const stories = storiesOf('VirtualList', {
  empty: { render: () => <Rows count={0} />, width: 480 },
  'hundred-thousand': { render: () => <Rows count={100_000} />, width: 480 },
  'hundred-thousand-middle': { render: () => <Rows count={100_000} scrollTo={50_000} />, width: 480 },
  ten: { render: () => <Rows count={10} />, width: 480 },
});
