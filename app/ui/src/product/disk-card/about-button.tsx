import type { JSX } from 'react';

import { Info } from '../../design/icons/icons';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { t } from '../../i18n';

export type AboutButtonProps = {
  // The id of the explanation the button opens.
  readonly controls: string;
  readonly expanded: boolean;
  // The part it is about, named in the button's label.
  readonly name: string;
  readonly onToggle: () => void;
};

// The "i" of a legend row: opens and closes what the part is, in the core's words.
export const AboutButton = ({ controls, expanded, name, onToggle }: AboutButtonProps): JSX.Element => (
  <IconButton
    aria-controls={controls}
    aria-expanded={expanded}
    icon={Info}
    label={t('ui.disk-card.about', { name: { kind: 'text', value: name } })}
    onClick={onToggle}
    size="small"
  />
);
