import { stories as storiesOf } from '../../../gallery/story';
import { Notice } from './notice';

const ignore = (): void => undefined;

export const stories = storiesOf('Notice', {
  danger: {
    render: () => (
      <Notice title="Nothing was removed" tone="danger">
        The Trash could not be reached. Your files are where they were.
      </Notice>
    ),
    width: 520,
  },
  info: {
    render: () => (
      <Notice title="Read-only settings" tone="info">
        The settings file was written by a newer version of Fresher, so it is not changed.
      </Notice>
    ),
    width: 520,
  },
  warning: {
    render: () => (
      <Notice title="The scan is partial" tone="warning">
        Some folders could not be read. Sizes ending in + may be larger.
      </Notice>
    ),
    width: 520,
  },
  'with-action': {
    render: () => (
      <Notice action={{ label: 'Show details', onClick: ignore }} title="The scan is partial" tone="warning">
        Some folders could not be read. Sizes ending in + may be larger.
      </Notice>
    ),
    width: 520,
  },
});
