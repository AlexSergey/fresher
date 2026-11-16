import type { JSX } from 'react';

import type { Forced } from '../forced';

import { stories as storiesOf } from '../../../gallery/story';
import { Select } from './select';

const OPTIONS = [
  { label: 'A week', value: 'week' },
  { label: 'A month', value: 'month' },
  { label: 'A quarter', value: 'quarter' },
] as const;

const ignore = (): void => undefined;

const select = (extra: Forced & { readonly disabled?: true }): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <Select label="Older than" onChange={ignore} options={OPTIONS} value="month" {...extra} />;
  };

const WIDTH = 240;

export const stories = storiesOf('Select', {
  closed: { render: select({}), width: WIDTH },
  disabled: { render: select({ disabled: true }), width: WIDTH },
  focus: { render: select({ 'data-focus': true }), width: WIDTH },
  hover: { render: select({ 'data-hover': true }), width: WIDTH },
});
