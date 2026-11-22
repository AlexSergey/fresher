import { type JSX, useEffect, useId, useState } from 'react';

import type { Insight, InsightPart, ModuleId } from '../../ipc/bindings.generated';

import { DangerBadge } from '../../design/data/danger-badge/danger-badge';
import { SizeText } from '../../design/data/size-text/size-text';
import { useAnnounce } from '../../design/feedback/announcer/announcer';
import { ChevronLeft } from '../../design/icons/icons';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { formatBytes } from '../../i18n/format-size';
import { levelName } from '../../product/finding-row/texts';
import { useNavigation } from '../../state/navigation';
import { copyText } from './copy-text';
import styles from './insight-screen.module.css';

export type InsightScreenProps = {
  readonly insight: Insight;
  readonly module: ModuleId;
};

// How long the words beside Copy stay.
const MESSAGE_MS = 4000;

const Size = ({ bytes }: { readonly bytes: number }): JSX.Element => (
  <SizeText accessibleText={formatBytes(bytes)} completeness="complete" text={formatBytes(bytes)} />
);

const PartRow = ({ part }: { readonly part: InsightPart }): JSX.Element => {
  const labelId = useId();
  const announce = useAnnounce();
  const [message, setMessage] = useState<null | string>(null);
  const { command } = part;

  useEffect(() => {
    if (message === null) {
      return;
    }

    const timer = setTimeout(() => {
      setMessage(null);
    }, MESSAGE_MS);

    return (): void => {
      clearTimeout(timer);
    };
  }, [message]);

  const copy = (line: string): void => {
    void copyText(line).then((copied) => {
      const text = t(copied ? 'ui.insight.copied' : 'ui.insight.copy-failed');

      announce(text);
      setMessage(text);
    });
  };

  return (
    <li aria-labelledby={labelId} className={styles.part}>
      <div className={styles.partHead}>
        <h2 className={styles.partLabel} id={labelId}>
          {tText(part.label)}
        </h2>
        <DangerBadge label={levelName(part.danger)} level={part.danger} size="small" />
        <Size bytes={part.size} />
      </div>
      <p className={styles.sentence}>{tText(part.detail)}</p>
      <p className={styles.sentence}>{tText(part.effect)}</p>
      {command === null ? null : (
        <div className={styles.command}>
          <code className={styles.line}>{command.line}</code>
          <Button
            aria-describedby={labelId}
            onClick={() => {
              copy(command.line);
            }}
            size="small"
          >
            {t('ui.insight.copy')}
          </Button>
          {message === null ? null : <span className={styles.message}>{message}</span>}
        </div>
      )}
    </li>
  );
};

// One insight in full: its figures, each part with its level, size and the command the user may copy and run by hand,
// then its notes. Nothing here is selectable for cleaning and the app never runs a command.
export const InsightScreen = ({ insight, module }: InsightScreenProps): JSX.Element => {
  const navigation = useNavigation();

  return (
    <>
      <Toolbar
        title={
          <Button
            icon={ChevronLeft}
            onClick={() => {
              navigation.navigate({ fromInsight: `${module}/${insight.identity}`, kind: 'scan' });
            }}
            variant="ghost"
          >
            {t('ui.insight.back')}
          </Button>
        }
      />
      <PageHeader subtitle={tText(insight.summary)} title={tText(insight.title)} />
      <div className={styles.content}>
        {insight.figures.length === 0 ? null : (
          <dl className={styles.figures}>
            {insight.figures.map(({ label, size }) => (
              <div className={styles.figure} key={label.key}>
                <dt>{tText(label)}</dt>
                <dd>
                  <Size bytes={size} />
                </dd>
              </div>
            ))}
          </dl>
        )}
        {insight.parts.length === 0 ? null : (
          <ul className={styles.parts}>
            {insight.parts.map((part) => (
              <PartRow key={part.label.key} part={part} />
            ))}
          </ul>
        )}
        {insight.notes.map((note) => (
          <p className={styles.note} key={note.key}>
            {tText(note)}
          </p>
        ))}
      </div>
    </>
  );
};
