import { stories as storiesOf } from '../../gallery/story';
import { PermissionCard } from './permission-card';

const ignore = (): void => undefined;
const HOW = ['Click Open System Settings.', 'Turn on Fresher in the list.', 'Come back to Fresher and scan again.'];
const TEXT = (
  <>
    <p>312 folders could not be read.</p>
    <p>Some folders could not be read because Fresher does not have Full Disk Access.</p>
  </>
);

export const stories = storiesOf('PermissionCard', {
  denied: {
    render: () => (
      <PermissionCard access="denied" how={HOW} onOpenSettings={ignore}>
        {TEXT}
      </PermissionCard>
    ),
    width: 560,
  },
  reopen: {
    render: () => (
      <PermissionCard access="denied" how={HOW} onOpenSettings={ignore} onReopen={ignore}>
        {TEXT}
      </PermissionCard>
    ),
    width: 560,
  },
  unknown: {
    render: () => (
      <PermissionCard access="unknown" how={HOW} onOpenSettings={ignore}>
        {TEXT}
      </PermissionCard>
    ),
    width: 560,
  },
});
