import { type JSX, useState } from 'react';

import type { Forced } from '../../primitives/forced';

import { stories as storiesOf } from '../../../gallery/story';
import { Tabs } from './tabs';

type Kind = 'groups' | 'items' | 'modules';

const TABS = [
  { label: 'Ignored groups', value: 'groups' },
  { label: 'Not looked for', value: 'modules' },
  { label: 'Ignored items', value: 'items' },
] as const;

const COUNTED = [
  { count: 1, label: 'Ignored groups', value: 'groups' },
  { count: 0, label: 'Not looked for', value: 'modules' },
  { count: 3, label: 'Ignored items', value: 'items' },
] as const;

const show = (
  initial: Kind,
  tabs: readonly { readonly count?: number; readonly label: string; readonly value: Kind }[] = TABS,
  extra: Forced = {},
): (() => JSX.Element) =>
  function Story(): JSX.Element {
    const [value, setValue] = useState<Kind>(initial);

    return (
      <Tabs label="Ignored" onChange={setValue} tabs={tabs} value={value} {...extra}>
        <p>Panel of {value}</p>
      </Tabs>
    );
  };

export const stories = storiesOf('Tabs', {
  counts: show('groups', COUNTED),
  first: show('groups'),
  focus: show('modules', TABS, { 'data-focus': true }),
  last: show('items'),
});
