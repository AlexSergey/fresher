import { stories as storiesOf } from '../../../gallery/story';
import { SizeText } from '../size-text/size-text';
import { StatTile } from './stat-tile';

export const stories = storiesOf('StatTile', {
  loading: { render: () => <StatTile label="Safe to remove" loading value={null} />, width: 200 },
  partial: {
    render: () => (
      <StatTile
        hint="Some folders could not be read"
        label="Safe to remove"
        value={<SizeText accessibleText="at least 61.4 GB" completeness="partial" size="title-3" text="61.4 GB" />}
      />
    ),
    width: 200,
  },
  value: {
    render: () => (
      <StatTile
        hint="in 6 modules"
        label="Safe to remove"
        value={<SizeText accessibleText="61.4 GB" completeness="complete" size="title-3" text="61.4 GB" />}
      />
    ),
    width: 200,
  },
});
