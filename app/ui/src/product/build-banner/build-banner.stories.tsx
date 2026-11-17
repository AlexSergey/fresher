import { stories as storiesOf } from '../../gallery/story';
import { BuildBanner } from './build-banner';

const WIDTH = 820;

export const stories = storiesOf('BuildBanner', {
  development: { render: () => <BuildBanner build={{ kind: 'development' }} />, width: WIDTH },
  sandbox: {
    render: () => (
      <BuildBanner build={{ kind: 'sandbox', path: '/Users/dev/sandboxes/trial', simulatedRunning: true }} />
    ),
    width: WIDTH,
  },
});
