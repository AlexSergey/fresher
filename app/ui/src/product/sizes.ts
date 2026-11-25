import type { SizeTextProps } from '../design/data/size-text/size-text';
import type { Sizes } from '../ipc/bindings.generated';

import { t, tText } from '../i18n';
import { formatBytes, formatSize } from '../i18n/format-size';

// What `SizeText` shows for a size the core measured: its allocated bytes, a lower bound with the core's reason, or a
// dash when it is not known (never 0). The caller adds the type step.
export const honestSize = ({ allocated, completeness }: Sizes): SizeTextProps => {
  switch (completeness) {
    case 'complete': {
      const text = formatBytes(allocated);

      return { accessibleText: text, completeness, text };
    }
    case 'partial':
      return {
        accessibleText: t('ui.size.at-least', { size: { kind: 'size', value: allocated } }),
        completeness,
        hint: tText({ args: {}, key: 'core.completeness.partial' }),
        text: formatBytes(allocated),
      };
    case 'unknown':
      return { accessibleText: t('ui.size.unknown'), completeness };
  }
};

// A size as one short text, for a chart's legend and its accessible name: `42 GB`, `42 GB+` or `—`.
export const sizeLabel = ({ allocated, completeness }: Sizes): string => formatSize(allocated, completeness);

const countFormat = new Intl.NumberFormat('en');

// A count as the dictionary formats one: `1,024`.
export const formatCount = (value: number): string => countFormat.format(Math.round(value));
