import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Badge } from '../../primitives/badge/badge';
import { Checkbox } from '../../primitives/checkbox/checkbox';
import { ListRow, type ListRowProps } from './list-row';

const ignore = (): void => undefined;

// A selectable row as the review screen shows one; the size is plain text until the size component exists.
const row = (props: Partial<ListRowProps>): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    return (
      <div role="list">
        <ListRow
          leading={
            <Checkbox
              checked={props.selected ?? false}
              disabled={props.disabled ?? false}
              hideLabel
              label="Select Xcode caches"
              onChange={ignore}
            />
          }
          subtitle="~/Library/Developer/Xcode/DerivedData"
          title="Xcode caches"
          trailing={
            <>
              <Badge>Rebuilt</Badge>
              12.4 GB
            </>
          }
          {...props}
        />
      </div>
    );
  },
  width: 480,
});

export const stories = storiesOf('ListRow', {
  disabled: row({ disabled: true }),
  focus: row({ 'data-focus': true }),
  hover: row({ 'data-hover': true }),
  rest: row({}),
  selected: row({ selected: true }),
  'selected-high': row({ edgeLevel: 'high', selected: true }),
});
