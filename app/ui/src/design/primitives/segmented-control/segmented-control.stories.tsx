import type { JSX } from 'react';

import type { Forced } from '../forced';

import { stories as storiesOf } from '../../../gallery/story';
import { SegmentedControl } from './segmented-control';

const TWO = [
  { label: 'Categories', value: 'categories' },
  { label: 'Folders', value: 'folders' },
] as const;

const THREE = [
  { label: 'Size', value: 'size' },
  { label: 'Name', value: 'name' },
  { label: 'Count', value: 'count' },
] as const;

const ignore = (): void => undefined;

const two = (
  value: (typeof TWO)[number]['value'],
  extra: Forced & { readonly disabled?: true } = {},
): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <SegmentedControl label="View" onChange={ignore} options={TWO} value={value} {...extra} />;
  };

const three = (
  value: (typeof THREE)[number]['value'],
  extra: Forced & { readonly disabled?: true } = {},
): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <SegmentedControl label="Sort by" onChange={ignore} options={THREE} value={value} {...extra} />;
  };

export const stories = storiesOf('SegmentedControl', {
  'three-count': three('count'),
  'three-disabled': three('name', { disabled: true }),
  'three-focus': three('name', { 'data-focus': true }),
  'three-name': three('name'),
  'three-size': three('size'),
  'two-categories': two('categories'),
  'two-folders': two('folders'),
});
