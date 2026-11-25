import { stories as storiesOf } from '../../gallery/story';
import { CategoryIconBadge } from './category-icon';

export const stories = storiesOf('CategoryIconBadge', {
  high: { render: () => <CategoryIconBadge icon="archive" level="high" />, width: 80 },
  low: { render: () => <CategoryIconBadge icon="download" level="low" />, width: 80 },
  medium: { render: () => <CategoryIconBadge icon="project" level="medium" size="small" />, width: 80 },
  safe: { render: () => <CategoryIconBadge icon="cache" level="safe" />, width: 80 },
});
