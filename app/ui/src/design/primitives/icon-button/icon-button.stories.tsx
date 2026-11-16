import { type JSX, useEffect, useRef } from 'react';

import { stories as storiesOf } from '../../../gallery/story';
import { RotateCw, X } from '../../icons/icons';
import { IconButton, type IconButtonProps } from './icon-button';
import styles from './icon-button.stories.module.css';

type State = Pick<IconButtonProps, 'data-focus' | 'data-hover' | 'data-pressed' | 'disabled'>;

// The three sizes of both variants in one state.
const row = (state: State): (() => JSX.Element) =>
  function Story(): JSX.Element {
    return (
      <div className={styles.row}>
        {(['small', 'medium', 'large'] as const).map((size) => (
          <IconButton icon={X} key={size} label="Close" size={size} {...state} />
        ))}
        {(['small', 'medium', 'large'] as const).map((size) => (
          <IconButton icon={RotateCw} key={size} label="Scan again" size={size} variant="secondary" {...state} />
        ))}
      </div>
    );
  };

// Focused once shown, as the keyboard would: its tooltip shows at once.
const WithTooltip = (): JSX.Element => {
  const stageRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    stageRef.current?.querySelector('button')?.focus();
  }, []);

  return (
    <div className={styles.stage} ref={stageRef}>
      <IconButton icon={RotateCw} label="Scan again" variant="secondary" />
    </div>
  );
};

export const stories = storiesOf('IconButton', {
  disabled: row({ disabled: true }),
  focus: row({ 'data-focus': true }),
  hover: row({ 'data-hover': true }),
  pressed: row({ 'data-pressed': true }),
  rest: row({}),
  'with-tooltip': { render: WithTooltip, width: 240 },
});
