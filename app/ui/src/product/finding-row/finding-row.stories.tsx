import type { JSX } from 'react';

import type { Finding } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  ADMIN_FINDING,
  EXTERNAL_FINDING,
  FINDING_DICTIONARY,
  HIGH_FINDING,
  LOCKED_FINDING,
  LONG_PATH_FINDING,
  LOW_FINDING,
  MEDIUM_FINDING,
  PROTECTED_FINDING,
  SAFE_FINDING,
} from '../../test/finding-fixtures';
import { FindingRow, type FindingRowProps, findingRowsClass } from './finding-row';
import styles from './finding-row.stories.module.css';

const MENU: FindingRowProps['menu'] = [
  {
    description: 'Hides it from the results and never removes it. You can restore it in Settings.',
    key: 'ignore-item',
    label: 'Ignore this item',
    onSelect: () => undefined,
  },
];

const ignore = (): void => undefined;

// One row in a list's frame, as a finding list shows it.
const row = (
  finding: Finding | undefined,
  props: Partial<FindingRowProps> = {},
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(FINDING_DICTIONARY);

    return (
      <div
        className={`${styles.frame ?? ''} ${findingRowsClass}`}
        data-menu-open={props.menuOpen === true ? '' : undefined}
        role="list"
      >
        <FindingRow finding={finding} onReveal={ignore} onToggle={ignore} selected={false} {...props} />
      </div>
    );
  },
  width: 640,
});

// One row in each removability: `safe` is removable.
export const stories = storiesOf('FindingRow', {
  external: row(EXTERNAL_FINDING),
  focus: row(SAFE_FINDING, { 'data-focus': true }),
  high: row(HIGH_FINDING),
  loading: row(undefined),
  locked: row(LOCKED_FINDING),
  'long-path': row(LONG_PATH_FINDING),
  low: row(LOW_FINDING),
  medium: row(MEDIUM_FINDING),
  'needs-administrator': row(ADMIN_FINDING),
  'protected-by-system': row(PROTECTED_FINDING),
  safe: row(SAFE_FINDING),
  selected: row(SAFE_FINDING, { selected: true }),
  'selected-high': row(HIGH_FINDING, { selected: true }),
  'with-empty-menu': row(SAFE_FINDING, { menu: [] }),
  'with-menu': row(SAFE_FINDING, { menu: MENU }),
  'with-menu-open': row(SAFE_FINDING, { menu: MENU, menuOpen: true }),
});
