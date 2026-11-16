import { type JSX, useState } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Button } from '../../primitives/button/button';
import { Dialog } from './dialog';
import styles from './dialog.stories.module.css';

const noop = (): void => undefined;

const FOLDERS = Array.from(
  { length: 30 },
  (_, index) => `~/Library/Developer/Xcode/DerivedData/Project-${String(index + 1)}`,
);

// A page behind the open dialog, so the backdrop's blur has something to blur.
const open = (long: boolean): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={styles.stage}>
        <p className={styles.behind}>The overview: 42.8 GB found in 12 categories.</p>
        <Button>Open the Trash</Button>
        <Dialog
          closeLabel="Close"
          description="The items go to the Trash; you can put them back until you empty it."
          footer={
            <>
              <Button variant="secondary">Cancel</Button>
              <Button variant="primary">Move to Trash</Button>
            </>
          }
          onClose={noop}
          open
          title={long ? 'Move 30 items to the Trash?' : 'Move 3 items to the Trash?'}
        >
          <ul className={styles.list}>
            {(long ? FOLDERS : FOLDERS.slice(0, 3)).map((folder) => (
              <li key={folder}>{folder}</li>
            ))}
          </ul>
        </Dialog>
      </div>
    );
  };

// Closed until its opener is pressed; Escape, Close and Cancel close it and the focus goes back to the opener.
const FromAButton = (): JSX.Element => {
  const [shown, setShown] = useState(false);
  const close = (): void => {
    setShown(false);
  };

  return (
    <div className={styles.stage}>
      <Button
        onClick={() => {
          setShown(true);
        }}
      >
        Review…
      </Button>
      <Dialog
        closeLabel="Close"
        footer={
          <Button onClick={close} variant="secondary">
            Cancel
          </Button>
        }
        onClose={close}
        open={shown}
        title="Move 3 items to the Trash?"
      >
        <ul className={styles.list}>
          {FOLDERS.slice(0, 3).map((folder) => (
            <li key={folder}>{folder}</li>
          ))}
        </ul>
      </Dialog>
    </div>
  );
};

export const stories = storiesOf('Dialog', {
  'from-a-button': { render: FromAButton, width: 1052 },
  'long-body': { render: open(true), width: 1052 },
  'short-body': { render: open(false), width: 1052 },
});
