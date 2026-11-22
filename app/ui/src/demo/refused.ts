import chrome from '../locales/en.json';
import { labelOf, nameOf } from './finder';

const IRREVERSIBLE = new Set(['ui.confirm.delete', 'ui.confirm.remove']);

// What a text says before its first placeholder.
const lead = (text: string): string => text.split('{')[0] ?? '';

// The texts of the app's own dictionary that a demo never presses: every permanent delete, and the removal by a tool
// (it cannot be put back). Read from the dictionary, so a rewording of the app is followed.
const REFUSED: readonly string[] = Object.entries(chrome)
  .filter(([key]) => key.startsWith('ui.') && (key.includes('permanent') || IRREVERSIBLE.has(key)))
  .map(([, text]) => lead(text))
  .filter((text) => text !== '');

// Whether the element is named, or labelled, by a refused text.
export const isRefused = (element: HTMLElement): boolean => {
  const names = [nameOf(element), labelOf(element)];

  return names.some((name) => REFUSED.some((text) => name.startsWith(text)));
};
