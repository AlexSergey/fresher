import type { Dictionary, Text, TextArgs, UiKey } from './types';

import chrome from '../locales/en.json';
import { translate } from './translate';

let coreDictionary: Dictionary = {};

// Keeps the merged dictionary of the core and its modules, read once from `get_dictionary`.
export const setCoreDictionary = (dictionary: Dictionary): void => {
  coreDictionary = dictionary;
};

// A text of the UI's own chrome; a missing key is a type error.
export const t = (key: UiKey, args: TextArgs = {}): string => translate(chrome, { args, key });

// A text the core sent.
export const tText = (text: Text): string => translate(coreDictionary, text);
