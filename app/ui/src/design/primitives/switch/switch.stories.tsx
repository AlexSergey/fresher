import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Switch, type SwitchProps } from './switch';

const ignore = (): void => undefined;

const control = (props: Partial<SwitchProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <Switch checked={false} label="Include the Trash" onChange={ignore} {...props} />;
  };

export const stories = storiesOf('Switch', {
  disabled: control({ disabled: true }),
  'disabled-on': control({ checked: true, disabled: true }),
  focus: control({ 'data-focus': true }),
  hover: control({ 'data-hover': true }),
  off: control({}),
  on: control({ checked: true }),
});
