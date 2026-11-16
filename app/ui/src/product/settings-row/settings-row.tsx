import { type JSX, useId } from 'react';

import type { SettingDescriptor, SettingProblem, SettingValue } from '../../ipc/bindings.generated';

import { PathText } from '../../design/data/path-text/path-text';
import { Icon } from '../../design/icons/icon';
import { CircleAlert, Lock } from '../../design/icons/icons';
import { SegmentedControl } from '../../design/primitives/segmented-control/segmented-control';
import { Select } from '../../design/primitives/select/select';
import { Switch } from '../../design/primitives/switch/switch';
import { t } from '../../i18n';
import { NumberField } from './number-field';
import styles from './settings-row.module.css';
import { optionText, problemText, settingText } from './texts';

export type SettingsRowProps = {
  readonly descriptor: SettingDescriptor;
  readonly onChange: (value: SettingValue) => void;
  readonly problem: null | SettingProblem;
  // The settings file cannot be changed (written by a newer version, broken or unreadable): every control is off.
  readonly readOnly: boolean;
  // The core's text of a change it refused: the row's error, in place of the stored value's problem.
  readonly refusal?: string;
  readonly value: SettingValue;
};

// A choice of this many options or fewer is a segmented control; more is a select.
const MOST_SEGMENTS = 3;

// One setting the core describes: its label and description from the dictionary, its control by kind, its stored
// value's problem as its error, and why it cannot be changed when the settings file is read-only.
export const SettingsRow = ({
  descriptor,
  onChange,
  problem,
  readOnly,
  refusal,
  value,
}: SettingsRowProps): JSX.Element => {
  const labelId = useId();
  const descriptionId = useId();
  const errorId = useId();
  const readOnlyId = useId();
  const label = settingText(descriptor.key, 'label');
  const error = refusal ?? (problem === null ? undefined : problemText(problem));
  // A number shows its problem in its own field; every other kind under its control.
  const rowError = descriptor.kind.kind === 'number' ? undefined : error;
  const describedBy = [descriptionId, rowError === undefined ? null : errorId, readOnly ? readOnlyId : null]
    .filter((id) => id !== null)
    .join(' ');

  return (
    <div
      aria-describedby={describedBy}
      aria-labelledby={labelId}
      className={styles.row}
      data-kind={descriptor.kind.kind}
      role="group"
    >
      <div className={styles.text}>
        <span className={styles.label} id={labelId}>
          {label}
        </span>
        <p className={styles.description} id={descriptionId}>
          {settingText(descriptor.key, 'description')}
        </p>
      </div>
      <div className={styles.control}>
        <Control
          descriptor={descriptor}
          error={error}
          label={label}
          onChange={onChange}
          readOnly={readOnly}
          value={value}
        />
      </div>
      {rowError === undefined ? null : (
        <p className={styles.error} data-error id={errorId}>
          <Icon className={styles.icon ?? ''} icon={CircleAlert} />
          {rowError}
        </p>
      )}
      {readOnly ? (
        <p className={styles.note} id={readOnlyId}>
          <Icon className={styles.icon ?? ''} icon={Lock} />
          {t('ui.settings-row.read-only')}
        </p>
      ) : null}
    </div>
  );
};

type ControlProps = Omit<SettingsRowProps, 'problem'> & { readonly error: string | undefined; readonly label: string };

const Control = ({ descriptor, error, label, onChange, readOnly, value }: ControlProps): JSX.Element => {
  // A value of another kind than the setting's cannot be shown: the default stands in for it.
  const shown = value.kind === descriptor.kind.kind ? value : descriptor.default;
  const kind = descriptor.kind;

  if (kind.kind === 'switch' && shown.kind === 'switch') {
    return (
      <Switch
        checked={shown.value}
        disabled={readOnly}
        hideLabel
        label={label}
        onChange={(checked) => {
          onChange({ kind: 'switch', value: checked });
        }}
      />
    );
  }

  if (kind.kind === 'number' && shown.kind === 'number') {
    return (
      <NumberField
        disabled={readOnly}
        label={label}
        max={kind.max}
        min={kind.min}
        onChange={(number) => {
          onChange({ kind: 'number', value: number });
        }}
        problem={error}
        unit={kind.unit}
        value={shown.value}
      />
    );
  }

  if (kind.kind === 'choice' && shown.kind === 'choice') {
    const options = kind.options.map((option) => ({ label: optionText(descriptor.key, option), value: option }));
    const choose = (option: string): void => {
      onChange({ kind: 'choice', value: option });
    };

    return options.length <= MOST_SEGMENTS ? (
      <SegmentedControl disabled={readOnly} label={label} onChange={choose} options={options} value={shown.value} />
    ) : (
      <Select disabled={readOnly} hideLabel label={label} onChange={choose} options={options} value={shown.value} />
    );
  }

  return <Folders folders={shown.kind === 'folders' ? shown.value : []} label={label} />;
};

// The folders, shown read-only: plan 005 adds editing.
const Folders = ({ folders, label }: { readonly folders: readonly string[]; readonly label: string }): JSX.Element =>
  folders.length === 0 ? (
    <p className={styles.empty}>{t('ui.settings-row.folders.empty')}</p>
  ) : (
    <ul aria-label={label} className={styles.folders}>
      {folders.map((folder) => (
        <li className={styles.folder} key={folder}>
          <PathText path={folder} />
        </li>
      ))}
    </ul>
  );
