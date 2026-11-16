import { stories as storiesOf } from '../../../gallery/story';
import { Skeleton } from './skeleton';

export const stories = storiesOf('Skeleton', {
  block: { render: () => <Skeleton shape="block" />, width: 320 },
  circle: () => <Skeleton shape="circle" />,
  line: { render: () => <Skeleton shape="line" />, width: 320 },
});
