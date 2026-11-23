import type { Locator } from '@playwright/test';

// The confirmation's High tick: a High plan cannot be confirmed until it is ticked.
const HIGH_TICK = 'I understand these may be the only copy of my data';

export const highTick = (dialog: Locator): Locator => dialog.getByRole('checkbox', { name: HIGH_TICK });

// Ticks it, as the user does before confirming a High plan.
export const tickHigh = async (dialog: Locator): Promise<void> => {
  await highTick(dialog).check();
};
