import { stories as storiesOf } from '../../../gallery/story';
import { ErrorState } from './error-state';

const ignore = (): void => undefined;

export const stories = storiesOf('ErrorState', {
  'with-try-again': {
    render: () => (
      <ErrorState
        action={{ label: 'Try again', onClick: ignore }}
        text="The folder ~/Library/Caches could not be read: permission denied."
        title="The list could not be loaded"
      />
    ),
    width: 480,
  },
});
