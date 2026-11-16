import { type JSX, useState } from 'react';

import { useAnimatedNumber } from '../../motion';
import { VisuallyHidden } from '../../primitives/visually-hidden/visually-hidden';
import { DURATION } from '../../tokens/motion';

export type AnimatedNumberProps = {
  // `slow` while a scan retargets the number (the default), `deliberate` for a first fill or the freed space.
  readonly duration?: 'deliberate' | 'slow';
  readonly format: (value: number) => string;
  readonly value: number;
};

// A number that counts toward its value while it grows; a value that falls (a new scan) jumps at once. Assistive
// technology reads only the final value, from a visually hidden copy.
export const AnimatedNumber = ({ duration = 'slow', format, value }: AnimatedNumberProps): JSX.Element => {
  const [run, setRun] = useState({ generation: 0, last: value });

  // A fall starts a new count: the counter mounts again at the new value, so it never counts down.
  if (value !== run.last) {
    setRun({ generation: value < run.last ? run.generation + 1 : run.generation, last: value });
  }

  return (
    <span data-numeric>
      <Counting duration={DURATION[duration]} format={format} key={run.generation} value={value} />
      <VisuallyHidden>{format(value)}</VisuallyHidden>
    </span>
  );
};

type CountingProps = {
  readonly duration: number;
  readonly format: (value: number) => string;
  readonly value: number;
};

const Counting = ({ duration, format, value }: CountingProps): JSX.Element => (
  <span aria-hidden="true">{format(useAnimatedNumber(value, { duration }))}</span>
);
