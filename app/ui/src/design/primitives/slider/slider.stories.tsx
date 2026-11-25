import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Slider, type SliderProps } from './slider';

const ignore = (): void => undefined;

const control = (props: Partial<SliderProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <Slider label="Old after, days" max={365} min={1} onChange={ignore} value={90} {...props} />;
  };

export const stories = storiesOf('Slider', {
  default: control({}),
  disabled: control({ disabled: true }),
  focus: control({ 'data-focus': true }),
});
