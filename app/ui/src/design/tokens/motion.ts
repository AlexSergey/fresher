// The motion tokens of motion.css, for the hooks that animate from JavaScript; a test keeps the two in step.

// Milliseconds.
export const DURATION = {
  base: 200,
  breath: 4000,
  deliberate: 800,
  fast: 120,
  move: 320,
  press: 80,
  slow: 480,
  sweep: 1600,
} as const;

export const EASING = {
  enter: 'cubic-bezier(0, 0, 0.2, 1)',
  exit: 'cubic-bezier(0.4, 0, 1, 1)',
  linear: 'linear',
  settle: 'cubic-bezier(0.34, 1.36, 0.64, 1)',
  standard: 'cubic-bezier(0.2, 0, 0, 1)',
} as const;

// The delay between two children entering, in milliseconds, and the most steps a list staggers.
export const STAGGER = {
  step: 30,
  steps: 8,
} as const;
