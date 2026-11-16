import { stories as storiesOf } from '../../gallery/story';
import { PermissionCard } from './permission-card';

const ignore = (): void => undefined;

export const stories = storiesOf('PermissionCard', {
  denied: { render: () => <PermissionCard access="denied" onLearnMore={ignore} />, width: 560 },
  unknown: { render: () => <PermissionCard access="unknown" onLearnMore={ignore} />, width: 560 },
});
