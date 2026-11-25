import { stories as storiesOf } from '../../../gallery/story';
import { SectionSeparator } from './section-separator';

export const stories = storiesOf('SectionSeparator', {
  default: { render: () => <SectionSeparator label="What you can remove" />, width: 640 },
});
