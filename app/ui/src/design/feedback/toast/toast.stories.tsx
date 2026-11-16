import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Toast, type ToastContent } from './toast';

const ignore = (): void => undefined;

// A toast as its toaster shows it, without the fixed region at the bottom of the window.
const shown = (content: ToastContent): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return <Toast {...content} closeLabel="Close" onClose={ignore} />;
  };

export const stories = storiesOf('Toast', {
  error: shown({ text: 'The Trash could not be reached.', tone: 'error' }),
  info: shown({ text: 'A scan is already running.', tone: 'info' }),
  success: shown({ text: 'Settings saved.', tone: 'success' }),
  'with-action': shown({
    action: { label: 'Show in Finder', onClick: ignore },
    text: 'Moved 3 items to the Trash.',
    tone: 'success',
  }),
});
