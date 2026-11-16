import type { JSX } from 'react';

import type { Finding } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  EXTERNAL_FINDING,
  FINDING_DICTIONARY,
  HIGH_FINDING,
  LONG_PATH_FINDING,
  LOW_FINDING,
  MEDIUM_FINDING,
  SAFE_FINDING,
} from '../../test/finding-fixtures';
import { FindingRow, type FindingRowProps, findingRowsClass } from './finding-row';
import styles from './finding-row.stories.module.css';

const ignore = (): void => undefined;

// One row in a list's frame, as a finding list shows it.
const row = (
  finding: Finding | undefined,
  props: Partial<FindingRowProps> = {},
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(FINDING_DICTIONARY);

    return (
      <div className={`${styles.frame ?? ''} ${findingRowsClass}`} role="list">
        <FindingRow finding={finding} onToggle={ignore} selected={false} {...props} />
      </div>
    );
  },
  width: 640,
});

export const stories = storiesOf('FindingRow', {
  external: row(EXTERNAL_FINDING),
  focus: row(SAFE_FINDING, { 'data-focus': true }),
  high: row(HIGH_FINDING),
  loading: row(undefined),
  'long-path': row(LONG_PATH_FINDING),
  low: row(LOW_FINDING),
  medium: row(MEDIUM_FINDING),
  safe: row(SAFE_FINDING),
  selected: row(SAFE_FINDING, { selected: true }),
  'selected-high': row(HIGH_FINDING, { selected: true }),
});
