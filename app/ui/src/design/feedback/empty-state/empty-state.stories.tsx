import { stories as storiesOf } from '../../../gallery/story';
import { ListChecks } from '../../icons/icons';
import { EmptyState } from './empty-state';

const ignore = (): void => undefined;

export const stories = storiesOf('EmptyState', {
  'with-action': {
    render: () => (
      <EmptyState
        action={{ label: 'Scan again', onClick: ignore }}
        icon={ListChecks}
        text="Nothing in this module takes space worth cleaning right now."
        title="Nothing to clean"
      />
    ),
    width: 480,
  },
});
