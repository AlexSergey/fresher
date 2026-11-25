const CONTROL = 'input, select, button, [tabindex]:not([tabindex="-1"])';

// Moves the focus to the control of the setting `key` and scrolls its row into view; nothing when its row is not on
// the page.
export const focusSetting = (key: string): void => {
  const row = document.querySelector(`[data-setting="${CSS.escape(key)}"]`);

  if (row === null) {
    return;
  }

  row.scrollIntoView({ block: 'center' });
  row.querySelector<HTMLElement>(CONTROL)?.focus();
};
