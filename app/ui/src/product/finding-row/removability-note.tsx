import type { JSX } from 'react';

import type { Finding, Removability } from '../../ipc/bindings.generated';

import { Icon } from '../../design/icons/icon';
import { FolderSearch, type IconShape, KeyRound, Lock, LockKeyhole } from '../../design/icons/icons';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { t } from '../../i18n';
import styles from './finding-row.module.css';
import { removabilityHow, removabilityName } from './texts';

export type RemovabilityNoteProps = {
  readonly finding: Finding;
  // "Show in Finder"; without it the row offers no button.
  readonly onReveal?: ((finding: Finding) => void) | undefined;
  // The finding's removability: any but `removable`.
  readonly removability: Kept;
};

type Kept = Exclude<Removability, 'removable'>;

const ICONS: Readonly<Record<Kept, IconShape>> = {
  locked: Lock,
  'needs-administrator': KeyRound,
  'protected-by-system': LockKeyhole,
};

// The user can free the space by hand: an administrator's item or a locked one, shown in Finder. macOS's own items
// are information only, and a tool's object has no place in Finder.
const canReveal = (finding: Finding, removability: Kept): boolean =>
  removability !== 'protected-by-system' && finding.subject.kind === 'path';

// Why the app keeps its hands off a finding: the core's short word with its icon, and its sentence on demand (the
// badge's tooltip, on hover or keyboard focus); "Show in Finder" where the user can remove it themselves.
export const RemovabilityNote = ({ finding, onReveal, removability }: RemovabilityNoteProps): JSX.Element => (
  <span className={styles.kept}>
    <Tooltip content={removabilityHow(removability)}>
      {(describedBy) => (
        // A stop of its own, so the keyboard and a screen reader reach the sentence.
        // eslint-disable-next-line jsx-a11y-x/no-noninteractive-tabindex -- the tooltip needs a focusable anchor
        <span aria-describedby={describedBy} className={styles.removability} tabIndex={0}>
          <Icon icon={ICONS[removability]} />
          {removabilityName(removability)}
        </span>
      )}
    </Tooltip>
    {onReveal !== undefined && canReveal(finding, removability) ? (
      <IconButton
        icon={FolderSearch}
        label={t('ui.finding-list.reveal')}
        onClick={() => {
          onReveal(finding);
        }}
        size="small"
      />
    ) : null}
  </span>
);
