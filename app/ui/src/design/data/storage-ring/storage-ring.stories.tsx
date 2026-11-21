import type { JSX } from 'react';

import type { Segment } from '../segment';

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

// A disk: the explained space by level, the parts of the system, and Free, which is left undrawn.
const DISK: readonly Segment[] = [
  { key: 'explained-safe', label: 'Safe', size: '38 GB', tone: 'safe', value: 38 },
  { key: 'explained-medium', label: 'Medium', size: '22 GB', tone: 'medium', value: 22 },
  { key: 'your-folders', label: 'Your folders', size: '90 GB', tone: 'your-folders', value: 90 },
  { key: 'mac-os', label: 'macOS', size: '30 GB', tone: 'mac-os', value: 30 },
  { key: 'swap-and-sleep', label: 'Swap and sleep', size: '12 GB', tone: 'swap-and-sleep', value: 12 },
  { key: 'other-volume', label: 'Backups', size: '20 GB', tone: 'other-volume', value: 20 },
  { key: 'disk-own', label: 'Disk itself', size: '8 GB', tone: 'disk-own', value: 8 },
  { key: 'free', label: 'Free', size: '80 GB', tone: 'free', value: 80 },
];

export const stories = storiesOf('StorageRing', {
  'disk-highlighted': () => (
    <StorageRing
      gapLabel="Free"
      highlighted="mac-os"
      label="Space on Macintosh HD"
      segments={DISK}
      state="done"
      total={300}
    >
      <Found text="220 GB" />
    </StorageRing>
  ),
  'disk-with-free-gap': () => (
    <StorageRing gapLabel="Free" label="Space on Macintosh HD" segments={DISK} state="done" total={300}>
      <Found text="220 GB" />
    </StorageRing>
  ),
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
