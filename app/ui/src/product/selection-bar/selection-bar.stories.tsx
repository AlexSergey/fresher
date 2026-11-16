import type { JSX } from 'react';

import { Announcer } from '../../design/feedback/announcer/announcer';
import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { SelectionBar, type SelectionBarProps } from './selection-bar';
import styles from './selection-bar.stories.module.css';

const ignore = (): void => undefined;

// The bar at the bottom of a screen-sized stage.
const shown = (
  summary: SelectionBarProps['summary'],
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(FINDING_DICTIONARY);

    return (
      <Announcer>
        <div className={styles.stage}>
          <SelectionBar onClear={ignore} onReview={ignore} summary={summary} />
        </div>
      </Announcer>
    );
  },
  width: 720,
});

export const stories = storiesOf('SelectionBar', {
  'many-high': shown({
    count: 128,
    highest: 'high',
    sizes: {
      allocated: 42_800_000_000,
      completeness: 'complete',
      logical: 42_800_000_000,
      reclaimable: 42_800_000_000,
    },
  }),
  'one-safe': shown({
    count: 1,
    highest: 'safe',
    sizes: { allocated: 1_500_000_000, completeness: 'complete', logical: 1_500_000_000, reclaimable: 1_500_000_000 },
  }),
  partial: shown({
    count: 12,
    highest: 'medium',
    sizes: { allocated: 7_300_000_000, completeness: 'partial', logical: 7_300_000_000, reclaimable: 7_300_000_000 },
  }),
});
