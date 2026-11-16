import type { JSX } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { Tooltip, type TooltipProps } from './tooltip';
import styles from './tooltip.stories.module.css';

const SHORT = 'Moves the items to the Trash';
const LONG =
  'Moves the selected items to the Trash. Nothing is deleted until you empty it, so you can put anything back.';

// The tooltip is drawn on <body>; the stage leaves it room inside the photographed frame.
const open = (props: Pick<TooltipProps, 'content' | 'placement'>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={styles.stage}>
        <Tooltip defaultOpen {...props}>
          {(describedBy) => (
            <button aria-describedby={describedBy} className={styles.target} type="button">
              Clean
            </button>
          )}
        </Tooltip>
      </div>
    );
  };

export const stories = storiesOf('Tooltip', {
  bottom: { render: open({ content: SHORT, placement: 'bottom' }), width: 320 },
  'long-text': { render: open({ content: LONG }), width: 320 },
  top: { render: open({ content: SHORT }), width: 320 },
});
