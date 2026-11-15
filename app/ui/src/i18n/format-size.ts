import type { Completeness } from './types';

const UNITS = ['kilobyte', 'megabyte', 'gigabyte', 'terabyte', 'petabyte'] as const;

const bytesFormat = new Intl.NumberFormat('en', { style: 'unit', unit: 'byte', unitDisplay: 'long' });

const unitFormats = UNITS.map(
  (unit) => new Intl.NumberFormat('en', { maximumFractionDigits: 1, style: 'unit', unit, unitDisplay: 'short' }),
);

// Bytes in decimal units, as Finder shows them: 1 kB is 1,000 bytes.
export const formatBytes = (bytes: number): string => {
  let value = bytes;
  let format = bytesFormat;

  for (const unitFormat of unitFormats) {
    if (value < 1000) {
      break;
    }
    value /= 1000;
    format = unitFormat;
  }

  return format.format(value);
};

// A size with how much of it is known: `42 GB`, `42 GB+` when some of it could not be read, `—` when it is unknown.
export const formatSize = (bytes: number, completeness: Completeness): string => {
  switch (completeness) {
    case 'complete':
      return formatBytes(bytes);
    case 'partial':
      return `${formatBytes(bytes)}+`;
    case 'unknown':
      return '—';
  }
};
