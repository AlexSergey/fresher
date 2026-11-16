import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Checkbox, type CheckboxProps } from './checkbox';

const LABEL = 'Xcode caches';
const DESCRIPTION = 'Build products of old projects. They come back the next time you build.';

const ignore = (): void => undefined;

const box = (props: Partial<CheckboxProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <Checkbox checked={false} label={LABEL} onChange={ignore} {...props} />;
  };

export const stories = storiesOf('Checkbox', {
  checked: box({ checked: true }),
  disabled: box({ disabled: true }),
  'disabled-checked': box({ checked: true, disabled: true }),
  focus: box({ 'data-focus': true }),
  hover: box({ 'data-hover': true }),
  mixed: box({ checked: 'mixed' }),
  unchecked: box({}),
  'with-description': { render: box({ checked: true, description: DESCRIPTION }), width: 360 },
});
