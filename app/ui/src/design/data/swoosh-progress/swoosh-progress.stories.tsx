import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { SwooshProgress } from './swoosh-progress';

const WIDTH = 520;

const Frame = ({ value }: { readonly value: null | number }): JSX.Element => (
  <div style={{ height: 142, width: WIDTH }}>
    <SwooshProgress label="Scanning" value={value} />
  </div>
);

export const stories = storiesOf('SwooshProgress', {
  'done-99': { render: () => <Frame value={0.99} />, width: WIDTH },
  indeterminate: { render: () => <Frame value={null} />, width: WIDTH },
  'part-62': { render: () => <Frame value={0.62} />, width: WIDTH },
  'start-0': { render: () => <Frame value={0} />, width: WIDTH },
});
