import type { Dictionary, Text, TextArg } from './types';

import { formatBytes } from './format-size';

const PLACEHOLDER = /\{([^{}]+)\}/g;

const pluralRules = new Intl.PluralRules('en');
const countFormat = new Intl.NumberFormat('en');
const dateFormat = new Intl.DateTimeFormat('en', { dateStyle: 'medium' });

const formatArg = (arg: TextArg): string => {
  switch (arg.kind) {
    case 'count':
      return countFormat.format(arg.value);
    case 'date':
      return dateFormat.format(new Date(arg.value * 1000));
    case 'path':
    case 'text':
      return arg.value;
    case 'size':
      return formatBytes(arg.value);
  }
};

// The keys to look up, in order: with a `count` arg its plural form, then `.other`, then the key itself.
const candidates = ({ args, key }: Text): readonly string[] => {
  const count = args.count;

  if (count?.kind !== 'count') {
    return [key];
  }

  return [`${key}.${pluralRules.select(count.value)}`, `${key}.other`, key];
};

// The text of `text` in `dictionary` with its placeholders filled. A missing key returns the key itself and an unknown
// placeholder stays as written, so a gap is visible, never blank.
export const translate = (dictionary: Dictionary, text: Text): string => {
  const template = candidates(text)
    .map((key) => dictionary[key])
    .find((found) => found !== undefined);

  if (template === undefined) {
    return text.key;
  }

  return template.replaceAll(PLACEHOLDER, (placeholder, name: string) => {
    const arg = text.args[name];

    return arg === undefined ? placeholder : formatArg(arg);
  });
};
