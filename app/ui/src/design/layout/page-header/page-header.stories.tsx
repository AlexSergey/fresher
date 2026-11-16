import { stories as storiesOf } from '../../../gallery/story';
import { Button } from '../../primitives/button/button';
import { PageHeader } from './page-header';

export const stories = storiesOf('PageHeader', {
  'title-only': { render: () => <PageHeader title="Overview" />, width: 640 },
  'with-subtitle-and-actions': {
    render: () => (
      <PageHeader
        actions={
          <>
            <Button variant="secondary">Open the Trash</Button>
            <Button variant="primary">Scan again</Button>
          </>
        }
        subtitle="Last scanned 5 minutes ago · 12 categories"
        title="Overview"
      />
    ),
    width: 640,
  },
});
