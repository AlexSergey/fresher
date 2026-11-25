import { stories as storiesOf } from '../../../gallery/story';
import { Tile } from './tile';

export const stories = storiesOf('Tile', {
  low: { render: () => <Tile tone="low">Cargo downloads</Tile>, width: 360 },
  medium: {
    render: () => (
      <Tile surface="medium" tone="medium">
        Old Xcode archives
      </Tile>
    ),
    width: 360,
  },
  safe: { render: () => <Tile tone="safe">Xcode caches</Tile>, width: 360 },
});
