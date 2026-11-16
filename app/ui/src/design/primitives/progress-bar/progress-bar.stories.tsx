import { stories as storiesOf } from '../../../gallery/story';
import { ProgressBar } from './progress-bar';

const WIDTH = 320;

export const stories = storiesOf('ProgressBar', {
  'done-100': { render: () => <ProgressBar label="Scanning" value={100} />, width: WIDTH },
  indeterminate: { render: () => <ProgressBar label="Scanning" value={null} />, width: WIDTH },
  'part-40': { render: () => <ProgressBar label="Scanning" value={40} />, width: WIDTH },
  'start-0': { render: () => <ProgressBar label="Scanning" value={0} />, width: WIDTH },
});
