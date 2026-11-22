import { type JSX, useId } from 'react';

import { t } from '../../i18n';
import { InsightCard } from '../../product/insight-card/insight-card';
import { useNavigation } from '../../state/navigation';
import { useCore } from '../../state/store';
import styles from './described-only.module.css';

// What the modules of the last scan described and do not offer to clean, under the zones: one card per insight, in
// the modules' order, each with Details. Nothing while a scan runs, for a hidden module, or when nothing was described.
export const DescribedOnly = (): JSX.Element | null => {
  const headingId = useId();
  const navigation = useNavigation();
  const scanning = useCore((state) => state.status.state === 'scanning');
  const finished = useCore((state) => state.modules);
  const overview = useCore((state) => state.overview);
  const hidden = (overview?.hiddenModules ?? []).map(({ module }) => module);
  const described = finished
    .filter(({ module }) => !hidden.includes(module))
    .flatMap(({ insights, module }) => insights.map((insight) => ({ insight, module })));

  if (scanning || described.length === 0) {
    return null;
  }

  return (
    <section aria-labelledby={headingId} className={styles.section}>
      <h2 className={styles.title} id={headingId}>
        {t('ui.screen.scan.described.title')}
      </h2>
      <p className={styles.text}>{t('ui.screen.scan.described.text')}</p>
      <ul className={styles.cards}>
        {described.map(({ insight, module }) => (
          <li
            className={styles.card}
            data-insight={`${module}/${insight.identity}`}
            key={`${module}/${insight.identity}`}
          >
            <InsightCard
              insight={insight}
              onDetails={() => {
                navigation.navigate({ identity: insight.identity, kind: 'insight', module });
              }}
            />
          </li>
        ))}
      </ul>
    </section>
  );
};
