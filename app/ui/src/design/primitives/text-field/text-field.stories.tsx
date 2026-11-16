import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { TextField, type TextFieldProps } from './text-field';

const ignore = (): void => undefined;

const field = (props: Partial<TextFieldProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <TextField
        description="Only caches untouched this long are listed."
        label="Older than (days)"
        onChange={ignore}
        placeholder="30"
        value=""
        {...props}
      />
    );
  };

const WIDTH = 320;

export const stories = storiesOf('TextField', {
  disabled: { render: field({ disabled: true, value: '30' }), width: WIDTH },
  empty: { render: field({}), width: WIDTH },
  filled: { render: field({ value: '45' }), width: WIDTH },
  focus: { render: field({ 'data-focus': true, value: '45' }), width: WIDTH },
  hover: { render: field({ 'data-hover': true }), width: WIDTH },
  invalid: { render: field({ error: 'Enter a whole number of days.', value: 'soon' }), width: WIDTH },
});
