import type { CSSProperties, JSX } from 'react';

import { Icon } from '../design/icons/icon';
import { OctagonAlert, Shield, ShieldCheck, TriangleAlert } from '../design/icons/icons';
import { stories as storiesOf } from './story';
import styles from './tokens.module.css';

// The design's tokens, so every later story stands on a photographed base.

const vars = (properties: Readonly<Record<string, string>>): CSSProperties => properties;

const PALETTE: readonly (readonly string[])[] = [
  ['--gray-1000', '--gray-950', '--gray-900', '--gray-850', '--gray-800', '--gray-750'],
  ['--gray-700', '--gray-600', '--gray-500', '--gray-400', '--gray-300', '--gray-100'],
  ['--iris-300', '--iris-400', '--iris-500', '--iris-600', '--iris-700'],
  ['--green-400', '--cyan-400', '--amber-400', '--red-400', '--red-600', '--red-700'],
  ['--part-your-folders', '--part-mac-os', '--part-swap-and-sleep', '--part-other-volume', '--part-disk-own'],
  ['--white-a4', '--white-a6', '--white-a10', '--white-a16', '--white-a36'],
];

const TYPE_STEPS = [
  'label',
  'caption',
  'footnote',
  'body',
  'body-strong',
  'callout',
  'title-3',
  'title-2',
  'title-1',
  'display',
  'hero',
] as const;

const SPACES = ['0-5', '1', '1-5', '2', '3', '4', '5', '6', '8', '10', '12', '16'] as const;

const RADII = ['xs', 'sm', 'md', 'lg', 'xl', 'full'] as const;

const LEVELS = [
  { icon: ShieldCheck, level: 'safe', name: 'Safe' },
  { icon: Shield, level: 'low', name: 'Low' },
  { icon: TriangleAlert, level: 'medium', name: 'Medium' },
  { icon: OctagonAlert, level: 'high', name: 'High' },
] as const;

const Palette = (): JSX.Element => (
  <div className={styles.rows}>
    {PALETTE.map((row) => (
      <div className={styles.swatches} key={row[0]}>
        {row.map((name) => (
          <figure className={styles.swatch} key={name}>
            <div className={styles.chip} style={vars({ '--swatch': `var(${name})` })} />
            <figcaption className={styles.name}>{name}</figcaption>
          </figure>
        ))}
      </div>
    ))}
  </div>
);

const TypeScale = (): JSX.Element => (
  <div className={styles.rows}>
    {TYPE_STEPS.map((step) => (
      <div className={styles.typeRow} key={step}>
        <span className={styles.name}>{step}</span>
        <span
          className={styles.sample}
          style={vars({
            '--sample-line': `var(--type-${step}-line)`,
            '--sample-size': `var(--type-${step}-size)`,
            '--sample-tracking': `var(--type-${step}-tracking)`,
            '--sample-weight': `var(--type-${step}-weight)`,
          })}
        >
          42.7 GB of caches
        </span>
      </div>
    ))}
  </div>
);

const Spacing = (): JSX.Element => (
  <div className={styles.rows}>
    {SPACES.map((space) => (
      <div className={styles.measureRow} key={space}>
        <span className={styles.name}>--space-{space}</span>
        <div className={styles.space} style={vars({ '--sample-space': `var(--space-${space})` })} />
      </div>
    ))}
  </div>
);

const Radii = (): JSX.Element => (
  <div className={styles.swatches}>
    {RADII.map((radius) => (
      <figure className={styles.swatch} key={radius}>
        <div className={styles.radius} style={vars({ '--sample-radius': `var(--radius-${radius})` })} />
        <figcaption className={styles.name}>--radius-{radius}</figcaption>
      </figure>
    ))}
  </div>
);

const Elevation = (): JSX.Element => (
  <div className={styles.cards}>
    {[1, 2, 3].map((level) => (
      <div
        className={styles.card}
        key={level}
        style={vars({
          '--card-border': `var(--elevation-${level}-border)`,
          '--card-shadow': `var(--elevation-${level}-shadow)`,
          '--card-surface': `var(--elevation-${level}-surface)`,
        })}
      >
        <span className={styles.name}>--elevation-{level}</span>
      </div>
    ))}
  </div>
);

const Levels = (): JSX.Element => (
  <div className={styles.rows}>
    {LEVELS.map(({ icon, level, name }) => (
      <div className={styles.levelRow} key={level}>
        <span
          className={styles.level}
          style={vars({
            '--level-border': `var(--level-${level}-border)`,
            '--level-fg': `var(--level-${level}-fg)`,
            '--level-tint': `var(--level-${level}-tint)`,
          })}
        >
          <Icon icon={icon} />
          {name}
        </span>
        <span
          className={styles.glow}
          style={vars({ '--level-fg': `var(--level-${level}-fg)`, '--level-glow': `var(--glow-${level})` })}
        >
          <Icon icon={icon} size={20} />
        </span>
      </div>
    ))}
  </div>
);

export const stories = storiesOf('Tokens', {
  elevation: { render: Elevation, width: 720 },
  levels: { render: Levels, surface: 'raised', width: 360 },
  palette: { render: Palette, width: 720 },
  radii: { render: Radii, width: 720 },
  spacing: { render: Spacing, width: 480 },
  'type-scale': { render: TypeScale, width: 720 },
});
