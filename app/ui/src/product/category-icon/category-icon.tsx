import type { JSX } from 'react';

import type { Level } from '../../design/data/danger-badge/danger-badge';
import type { CategoryIcon } from '../../ipc/bindings.generated';

import {
  AppWindow,
  Archive,
  Clock,
  Code,
  Cpu,
  Download,
  File,
  FileText,
  Folder,
  Globe,
  Hammer,
  type IconShape,
  Layers,
  Smartphone,
  SquareCode,
} from '../../design/icons/icons';
import styles from './category-icon.module.css';

export type CategoryIconBadgeProps = {
  readonly icon: CategoryIcon;
  readonly level: Level;
  readonly size?: 'large' | 'small';
};

// The shape the UI draws for each icon key the core names.
export const CATEGORY_ICONS: Readonly<Record<CategoryIcon, IconShape>> = {
  app: AppWindow,
  archive: Archive,
  browser: Globe,
  build: Hammer,
  cache: Layers,
  device: Smartphone,
  download: Download,
  editor: SquareCode,
  generic: File,
  log: FileText,
  model: Cpu,
  project: Folder,
  temporary: Clock,
  toolchain: Code,
};

const GLYPH: Readonly<Record<'large' | 'small', number>> = { large: 20, small: 18 };

// A category's icon in a square tinted by its danger level.
export const CategoryIconBadge = ({ icon, level, size = 'large' }: CategoryIconBadgeProps): JSX.Element => {
  const Shape = CATEGORY_ICONS[icon];

  return (
    <span aria-hidden="true" className={styles.badge} data-level={level} data-size={size}>
      <Shape color="currentColor" focusable="false" size={GLYPH[size]} strokeWidth={2} />
    </span>
  );
};
