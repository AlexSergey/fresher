import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Button } from '../../primitives/button/button';
import { STORY_SEGMENTS, STORY_TOTAL } from '../chart-stories';
import { SizeText } from '../size-text/size-text';
import { StorageRing } from './storage-ring';
import styles from './storage-ring.stories.module.css';

const Found = ({ partial = false, text }: { readonly partial?: boolean; readonly text: string }): JSX.Element => (
  <div className={styles.centre}>
    <span className={styles.caption}>Found</span>
    {partial ? (
      <SizeText accessibleText={`at least ${text}`} completeness="partial" size="display" text={text} />
    ) : (
      <SizeText accessibleText={text} completeness="complete" size="display" text={text} />
    )}
  </div>
);

const FIRST_FOUR = STORY_SEGMENTS.slice(0, 4);

export const stories = storiesOf('StorageRing', {
  empty: () => (
    <StorageRing label="Space found" segments={[]} state="empty" total={0}>
      <Button variant="primary">Scan</Button>
    </StorageRing>
  ),
  'full-1-segment': () => (
    <StorageRing label="Space found" segments={STORY_SEGMENTS.slice(0, 1)} state="done" total={382}>
      <Found text="38.2 GB" />
    </StorageRing>
  ),
  'full-12-segments': () => (
    <StorageRing label="Space found" segments={STORY_SEGMENTS} state="done" total={STORY_TOTAL}>
      <Found text="102.6 GB" />
    </StorageRing>
  ),
  partial: () => (
    <StorageRing label="Space found" segments={FIRST_FOUR} state="partial" total={1200}>
      <Found partial text="84.1 GB" />
    </StorageRing>
  ),
  'partly-filled': () => (
    <StorageRing label="Space found" segments={FIRST_FOUR} state="done" total={1200}>
      <Found text="84.1 GB" />
    </StorageRing>
  ),
  scanning: () => (
    <StorageRing label="Space found" segments={STORY_SEGMENTS.slice(0, 2)} state="scanning" total={1200}>
      <Found text="59.8 GB" />
    </StorageRing>
  ),
});
