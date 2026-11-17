import type { CSSProperties, JSX, ReactNode } from 'react';

import type { CategoryTotal, DangerLevel, Sizes } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { useSpotlightProps } from '../../design/effects/spotlight/spotlight';
import { Icon } from '../../design/icons/icon';
import { type IconShape, OctagonAlert, Shield, ShieldCheck, TriangleAlert } from '../../design/icons/icons';
import { Surface } from '../../design/layout/surface/surface';
import { Checkbox } from '../../design/primitives/checkbox/checkbox';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './category-card.module.css';

export type CategoryCardProps = {
  // A slot for the card's own actions: the ignore list's "Ignore" will go here.
  readonly actions?: ReactNode;
  readonly onOpen: () => void;
  readonly onToggle: () => void;
  // Whether the whole category is picked for removal; `mixed` when only some of its findings are.
  readonly selected: 'mixed' | boolean;
  // The category's part of everything found, 0 to 1.
  readonly share: number;
  readonly total: CategoryTotal;
  // The part of the category the by-meaning view lists under apps, which the card's size leaves out; the selection
  // still takes the whole category, so the card says how much more it holds.
  readonly underApps?: Sizes | undefined;
};

// The icons escalate in shape with the level, as the danger badge's do.
const ICONS: Readonly<Record<DangerLevel, IconShape>> = {
  high: OctagonAlert,
  low: Shield,
  medium: TriangleAlert,
  safe: ShieldCheck,
};

const textOf = (key: string): string => tText({ args: {}, key });

const clamp = (share: number): number => Math.min(Math.max(share, 0), 1);

// One category the scan found: its level's icon, its name and what it is, its level, size and count, and a bar of its
// share along the bottom. The card opens the category; the checkbox in its corner selects it whole.
export const CategoryCard = ({
  actions,
  onOpen,
  onToggle,
  selected,
  share,
  total,
  underApps,
}: CategoryCardProps): JSX.Element => {
  const spotlight = useSpotlightProps();
  const { category, count, highestDanger: level, sizes } = total;
  const name = textOf(`${category}.name`);
  const what = textOf(`${category}.what`);
  const drawn = clamp(share);

  return (
    <div className={styles.card} data-card data-level={level} data-selected={selected === false ? undefined : selected}>
      <Surface
        className={`${spotlight.className} ${styles.surface ?? ''}`}
        elevation={1}
        padding="none"
        ref={spotlight.ref}
      >
        {/* Named by its content: everything the card shows is in what it reads (WCAG 2.5.3, Label in Name). */}
        <button className={styles.open} onClick={onOpen} type="button">
          <span className={styles.icon}>
            <Icon icon={ICONS[level]} size={20} />
          </span>
          <span className={styles.text}>
            <span className={styles.name}>
              {/* Read once, as "Open …"; the visible name is that same text. */}
              <span className="u-visually-hidden">
                {t('ui.category-card.open', { name: { kind: 'text', value: name } })}
              </span>
              <span aria-hidden="true">{name}</span>
            </span>
            <Tooltip content={what} repeatsName>
              {() => <span className={styles.what}>{what}</span>}
            </Tooltip>
            <span className={styles.meta}>
              <DangerBadge label={textOf(`core.danger.${level}.name`)} level={level} size="small" />
              <span className={styles.figures}>
                <SizeText {...honestSize(sizes)} size="title-3" />
                <span className={styles.count}>
                  {t('ui.category-card.count', { count: { kind: 'count', value: count } })}
                </span>
              </span>
            </span>
            {underApps === undefined ? null : (
              <span className={styles.under}>
                {t('ui.category-card.under-apps', { size: { kind: 'size', value: underApps.allocated } })}
              </span>
            )}
          </span>
        </button>
        <span className={styles.check}>
          <Checkbox
            checked={selected}
            hideLabel
            label={t('ui.category-card.select', { name: { kind: 'text', value: name } })}
            onChange={onToggle}
          />
        </span>
        {actions === undefined ? null : <div className={styles.actions}>{actions}</div>}
        {drawn > 0 ? (
          <span aria-hidden="true" className={styles.share} data-share style={{ '--share': drawn } as CSSProperties} />
        ) : null}
      </Surface>
    </div>
  );
};
