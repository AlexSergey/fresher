import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { TreeRow } from './tree-row';

const ignore = (): void => undefined;

const single = (expanded: boolean) =>
  function Story(): JSX.Element {
    return (
      <div role="list">
        <TreeRow
          expanded={expanded}
          onExpandedChange={ignore}
          subtitle="3 folders"
          title="Xcode"
          toggleLabel="Show the items of Xcode"
          trailing="18.2 GB"
        />
      </div>
    );
  };

const TwoLevels = (): JSX.Element => (
  <div role="list">
    <TreeRow
      expanded
      onExpandedChange={ignore}
      title="Xcode"
      toggleLabel="Show the items of Xcode"
      trailing="18.2 GB"
    />
    <TreeRow
      expanded
      level={1}
      onExpandedChange={ignore}
      title="DerivedData"
      toggleLabel="Show the items of DerivedData"
      trailing="12.4 GB"
    />
    <TreeRow
      expanded={false}
      level={2}
      onExpandedChange={ignore}
      title="Fresher-abcdefghijklmnop"
      toggleLabel="Show the items of Fresher-abcdefghijklmnop"
      trailing="4.1 GB"
    />
    <TreeRow
      expanded={false}
      level={1}
      onExpandedChange={ignore}
      title="Archives"
      toggleLabel="Show the items of Archives"
      trailing="5.8 GB"
    />
  </div>
);

export const stories = storiesOf('TreeRow', {
  collapsed: { render: single(false), width: 480 },
  expanded: { render: single(true), width: 480 },
  'two-levels': { render: TwoLevels, width: 480 },
});
