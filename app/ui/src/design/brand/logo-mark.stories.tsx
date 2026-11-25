import { stories as storiesOf } from '../../gallery/story';
import { LogoMark } from './logo-mark';

export const stories = storiesOf('LogoMark', {
  large: { render: () => <LogoMark size={96} />, width: 120 },
  sidebar: { render: () => <LogoMark />, width: 60 },
});
