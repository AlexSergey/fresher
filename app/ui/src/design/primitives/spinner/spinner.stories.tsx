import { stories as storiesOf } from '../../../gallery/story';
import { Spinner } from './spinner';

export const stories = storiesOf('Spinner', {
  medium: () => <Spinner label="Loading" />,
  small: () => <Spinner label="Loading" size="small" />,
});
