import { stories as storiesOf } from '../../../gallery/story';
import { Badge } from './badge';

export const stories = storiesOf('Badge', {
  accent: () => <Badge tone="accent">New</Badge>,
  count: () => <Badge count={128} label="128 items" />,
  'count-accent': () => <Badge count={42} label="42 items" tone="accent" />,
  'count-capped': () => <Badge count={12_408} label="12,408 items" />,
  neutral: () => <Badge>Read-only</Badge>,
});
