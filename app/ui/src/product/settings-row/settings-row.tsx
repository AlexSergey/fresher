import { type JSX, useId } from 'react';

import type { SettingDescriptor, SettingProblem, SettingValue } from '../../ipc/bindings.generated';

import { PathText } from '../../design/data/path-text/path-text';
import { Icon } from '../../design/icons/icon';
import { CircleAlert, Lock, Plus, X } from '../../design/icons/icons';
import { Button } from '../../design/primitives/button/button';
import { IconButton } from '../../design/primitives/icon-button/icon-button';
import { SegmentedControl } from '../../design/primitives/segmented-control/segmented-control';
import { Select } from '../../design/primitives/select/select';
import { Switch } from '../../design/primitives/switch/switch';
import { t } from '../../i18n';
import { NumberField } from './number-field';
import styles from './settings-row.module.css';
import { optionText, problemText, settingText } from './texts';

export type SettingsRowProps = {
  readonly descriptor: SettingDescriptor;
  // A folder list's "Add folder…": the screen asks the system's folder picker, then sends the new list.
  readonly onAddFolder?: () => void;
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
  onAddFolder,
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
          {...(onAddFolder === undefined ? {} : { onAddFolder })}
        />
      </div>
      {readOnly || sameValue(value, descriptor.default) ? null : (
        <div className={styles.reset}>
          <Button
            onClick={() => {
              onChange(descriptor.default);
            }}
            size="small"
            variant="ghost"
          >
            {t('ui.settings-row.reset')}
          </Button>
        </div>
      )}
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

const Control = ({ descriptor, error, label, onAddFolder, onChange, readOnly, value }: ControlProps): JSX.Element => {
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

  if (shown.kind === 'folders') {
    return (
      <Folders
        folders={shown.value}
        label={label}
        onRemove={(removed) => {
          onChange({ kind: 'folders', value: shown.value.filter((folder) => folder !== removed) });
        }}
        readOnly={readOnly}
        {...(onAddFolder === undefined ? {} : { onAdd: onAddFolder })}
      />
    );
  }

  // The ignore lists have a screen part of their own; as a row they are listed only.
  return <Folders folders={shown.kind === 'ids' || shown.kind === 'paths' ? shown.value : []} label={label} readOnly />;
};

type FoldersProps = {
  readonly folders: readonly string[];
  readonly label: string;
  readonly onAdd?: () => void;
  readonly onRemove?: (folder: string) => void;
  readonly readOnly: boolean;
};

// The folders, each with Remove, and "Add folder…" under them when the screen can pick one.
const Folders = ({ folders, label, onAdd, onRemove, readOnly }: FoldersProps): JSX.Element => (
  <div className={styles.folderList}>
    {folders.length === 0 ? (
      <p className={styles.empty}>{t('ui.settings-row.folders.empty')}</p>
    ) : (
      <ul aria-label={label} className={styles.folders}>
        {folders.map((folder) => (
          <li className={styles.folder} key={folder}>
            <PathText path={folder} />
            {onRemove === undefined || readOnly ? null : (
              <IconButton
                icon={X}
                label={t('ui.settings-row.folders.remove', { folder: { kind: 'path', value: folder } })}
                onClick={() => {
                  onRemove(folder);
                }}
                size="small"
                variant="ghost"
              />
            )}
          </li>
        ))}
      </ul>
    )}
    {onAdd === undefined ? null : (
      <div>
        <Button disabled={readOnly} icon={Plus} onClick={onAdd} size="small" variant="secondary">
          {t('ui.settings-row.folders.add')}
        </Button>
      </div>
    )}
  </div>
);

// Two values are the same when they are of one kind and hold the same things.
const sameValue = (one: SettingValue, other: SettingValue): boolean => JSON.stringify(one) === JSON.stringify(other);
