import { type JSX, useId } from 'react';

import type { CategoryId, LeftOutCount, SettingValue } from '../../ipc/bindings.generated';

import { Tile } from '../../design/layout/tile/tile';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { LeftOutLines } from '../../product/left-out-lines/left-out-lines';
import { useNavigation } from '../../state/navigation';
import { useCore } from '../../state/store';
import styles from './left-out-only.module.css';

export type LeftOutOnlyProps = {
  // The categories the overview already shows as cards.
  readonly cards: readonly CategoryId[];
};

type Row = { readonly category: CategoryId; readonly counts: readonly LeftOutCount[] };

const NONE: readonly string[] = [];

// The ids an ignore list holds; none until the settings are loaded.
const idsOf = (value: SettingValue | undefined): readonly string[] => (value?.kind === 'ids' ? value.value : NONE);

// The categories of the last scan that the modules left out whole: left-out counts but no card. A hidden module's and
// an ignored category's left-outs, kept from the scan's events, leave with their findings.
const rowsOf = (
  finished: readonly { readonly leftOut: readonly LeftOutCount[]; readonly module: string }[],
  cards: readonly CategoryId[],
  hidden: readonly string[],
  ignored: readonly string[],
): readonly Row[] => {
  const rows = new Map<CategoryId, LeftOutCount[]>();

  for (const { leftOut, module } of finished) {
    if (hidden.includes(module)) {
      continue;
    }
    for (const count of leftOut) {
      if (!cards.includes(count.category) && !ignored.includes(count.category)) {
        rows.set(count.category, [...(rows.get(count.category) ?? []), count]);
      }
    }
  }

  return [...rows].map(([category, counts]) => ({ category, counts }));
};

const LeftOutRow = ({ category, counts }: Row): JSX.Element => {
  const navigation = useNavigation();
  const nameId = useId();

  return (
    <li aria-labelledby={nameId} className={styles.row}>
      <Tile>
        <div className={styles.head}>
          <h3 className={styles.name} id={nameId}>
            {tText({ args: {}, key: `${category}.name` })}
          </h3>
          <span className={styles.details}>
            <Button
              aria-describedby={nameId}
              onClick={() => {
                navigation.navigate({ category, kind: 'category' });
              }}
              size="small"
              variant="ghost"
            >
              {t('ui.category-card.details')}
            </Button>
          </span>
        </div>
        <LeftOutLines counts={counts} />
      </Tile>
    </li>
  );
};

// What the last scan found but left out whole, under the groups: a heading like theirs, then one tile per category,
// with its lines and Details.
// Nothing before a scan, during one (its totals arrive after its left-outs, so a category with findings would show
// here first), or when every category with left-outs has a card.
export const LeftOutOnly = ({ cards }: LeftOutOnlyProps): JSX.Element | null => {
  const headingId = useId();
  const scanning = useCore((state) => state.status.state === 'scanning');
  const finished = useCore((state) => state.modules);
  const overview = useCore((state) => state.overview);
  const ignored = useCore((state) => state.settings?.values['core.ignore.categories']);
  const hidden = (overview?.hiddenModules ?? []).map(({ module }) => module);
  const rows = rowsOf(finished, cards, hidden, idsOf(ignored));

  if (scanning || rows.length === 0) {
    return null;
  }

  return (
    <section aria-labelledby={headingId} className={styles.section}>
      <div className={styles.heading}>
        <h2 className={styles.title} id={headingId}>
          {t('ui.screen.scan.left-out.title')}
        </h2>
        <p className={styles.text}>
          {t('ui.screen.scan.group-line', { line: { kind: 'text', value: t('ui.screen.scan.left-out.text') } })}
        </p>
      </div>
      <ul className={styles.rows}>
        {rows.map((row) => (
          <LeftOutRow category={row.category} counts={row.counts} key={row.category} />
        ))}
      </ul>
    </section>
  );
};
