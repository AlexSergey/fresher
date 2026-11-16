import { stories as storiesOf } from '../../../gallery/story';
import { PathText } from './path-text';

export const stories = storiesOf('PathText', {
  long: {
    render: () => (
      <PathText path="~/Library/Developer/Xcode/DerivedData/Fresher-abcdefghijklmnop/Build/Intermediates.noindex/Index.db" />
    ),
    width: 320,
  },
  short: { render: () => <PathText path="~/Library/Caches" />, width: 320 },
});
