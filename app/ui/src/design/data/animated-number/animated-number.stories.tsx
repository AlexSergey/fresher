import { stories as storiesOf } from '../../../gallery/story';
import { AnimatedNumber } from './animated-number';

const format = (value: number): string => `${(value / 10).toFixed(1)} GB`;

// With motion off, the number shows its final value at once.
export const stories = storiesOf('AnimatedNumber', {
  'final-value': () => <AnimatedNumber format={format} value={1284} />,
});
