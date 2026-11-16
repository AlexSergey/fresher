import { stories as storiesOf } from '../../../gallery/story';
import { ScanSweep } from './scan-sweep';
import styles from './scan-sweep.stories.module.css';

// With motion off, the sweep stands at its first frame: its head at the top.
export const stories = storiesOf('ScanSweep', {
  frozen: () => (
    <div className={styles.square}>
      <ScanSweep active />
    </div>
  ),
});
