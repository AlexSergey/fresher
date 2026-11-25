import type { JSX } from 'react';

import { fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { SettingDescriptor, SettingProblem, SettingValue } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { SettingsRow } from './settings-row';

const SWITCH: SettingDescriptor = {
  default: { kind: 'switch', value: true },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.enabled',
  kind: { kind: 'switch' },
};
const NUMBER: SettingDescriptor = {
  default: { kind: 'number', value: 30 },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.min-age',
  kind: { kind: 'number', max: 365, min: 1, unit: 'days' },
};
const FEW_CHOICES: SettingDescriptor = {
  default: { kind: 'choice', value: 'trash' },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.method',
  kind: { kind: 'choice', options: ['trash', 'permanent'] },
};
const MANY_CHOICES: SettingDescriptor = {
  default: { kind: 'choice', value: 'week' },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.every',
  kind: { kind: 'choice', options: ['day', 'week', 'month', 'year'] },
};
const FOLDERS: SettingDescriptor = {
  default: { kind: 'folders', value: [] },
  effect: 'next-scan',
  group: 'module',
  key: 'demo.roots',
  kind: { kind: 'folders' },
};

type RowOptions = {
  readonly onChange?: (value: SettingValue) => void;
  readonly problem?: null | SettingProblem;
  readonly readOnly?: boolean;
  readonly refusal?: string;
  readonly value?: SettingValue;
};

const renderRow = (
  descriptor: SettingDescriptor,
  { onChange = vi.fn(), problem = null, readOnly = false, refusal, value = descriptor.default }: RowOptions = {},
): ReturnType<typeof render> =>
  render(
    <SettingsRow
      descriptor={descriptor}
      onChange={onChange}
      problem={problem}
      readOnly={readOnly}
      {...(refusal === undefined ? {} : { refusal })}
      value={value}
    />,
  );

beforeEach(() => {
  setCoreDictionary({
    'demo.enabled.description': 'Look for the demo findings.',
    'demo.enabled.label': 'Demo module',
    'demo.every.description': 'How often to remind you.',
    'demo.every.label': 'Remind me',
    'demo.every.options.day': 'Every day',
    'demo.every.options.month': 'Every month',
    'demo.every.options.week': 'Every week',
    'demo.method.description': 'Where removed items go.',
    'demo.method.label': 'Remove to',
    'demo.method.options.permanent': 'Delete permanently',
    'demo.method.options.trash': 'Trash',
    'demo.min-age.description': 'Projects untouched this long count as stale.',
    'demo.min-age.label': 'Stale after',
    'demo.roots.description': 'Where your projects live.',
    'demo.roots.label': 'Project folders',
  });
});

afterEach(() => {
  setCoreDictionary({});
});

describe('SettingsRow', () => {
  describe('negative cases', () => {
    it('a number outside its limits is not sent', () => {
      const onChange = vi.fn();
      renderRow(NUMBER, { onChange });
      const field = screen.getByRole('spinbutton', { name: 'Stale after' });

      fireEvent.change(field, { target: { value: '400' } });
      fireEvent.change(field, { target: { value: '0' } });
      fireEvent.change(field, { target: { value: '' } });
      fireEvent.change(field, { target: { value: '2.5' } });

      expect(onChange).not.toHaveBeenCalled();
      expect(field).toHaveAttribute('aria-invalid', 'true');
      expect(screen.getByText('Enter a whole number from 1 to 365.')).toBeInTheDocument();
    });

    it('a read-only file disables both the slider and the field', () => {
      renderRow(NUMBER, { readOnly: true });

      expect(screen.getByRole('slider', { name: 'Stale after, days' })).toBeDisabled();
      expect(screen.getByRole('spinbutton', { name: 'Stale after' })).toBeDisabled();
    });

    it('a typed number outside the limits is not sent and the slider keeps the stored value', () => {
      const onChange = vi.fn();
      renderRow(NUMBER, { onChange, value: { kind: 'number', value: 90 } });

      fireEvent.change(screen.getByRole('spinbutton', { name: 'Stale after' }), { target: { value: '400' } });

      expect(onChange).not.toHaveBeenCalled();
      expect(screen.getByRole('slider', { name: 'Stale after, days' })).toHaveValue('90');
    });

    it('a read-only settings file disables every control and says why', () => {
      const why = 'Read-only: this version of Fresher cannot change the settings file safely.';

      for (const descriptor of [SWITCH, NUMBER, FEW_CHOICES, MANY_CHOICES, FOLDERS]) {
        const { unmount } = renderRow(descriptor, {
          readOnly: true,
          value: descriptor === FOLDERS ? { kind: 'folders', value: ['~/Projects'] } : descriptor.default,
        });

        for (const control of [
          ...screen.queryAllByRole('switch'),
          ...screen.queryAllByRole('spinbutton'),
          ...screen.queryAllByRole('combobox'),
          ...screen.queryAllByRole('radio'),
        ]) {
          expect(control).toBeDisabled();
        }

        expect(screen.getByText(why)).toBeInTheDocument();
        unmount();
      }
    });

    it("a refused change shows the core's text instead of the stored value's problem", () => {
      const refusal = 'Your settings could not be saved.';

      renderRow(SWITCH, { problem: { key: 'demo.enabled', kind: 'wrong-type' }, refusal });
      renderRow(NUMBER, { refusal });

      expect(screen.getByRole('group', { name: 'Demo module' })).toHaveAccessibleDescription(
        `Look for the demo findings. ${refusal}`,
      );
      expect(screen.queryByText(/wrong kind/u)).not.toBeInTheDocument();
      expect(screen.getByRole('spinbutton', { name: 'Stale after' })).toHaveAttribute('aria-invalid', 'true');
    });

    it('a read-only switch does not change', () => {
      const onChange = vi.fn();
      renderRow(SWITCH, { onChange, readOnly: true });

      fireEvent.click(screen.getByRole('switch', { name: 'Demo module' }));

      expect(onChange).not.toHaveBeenCalled();
    });

    it('says so when a folder list is empty', () => {
      renderRow(FOLDERS);

      expect(screen.getByText('No folders')).toBeInTheDocument();
      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });

    it('shows a choice the dictionary does not name as it is written', () => {
      renderRow(MANY_CHOICES);

      expect(screen.getByRole('option', { name: 'year' })).toBeInTheDocument();
    });

    it('has no error and no read-only note when all is well', () => {
      renderRow(SWITCH);

      expect(screen.queryByText(/Read-only/u)).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- an error has no role of its own
      expect(document.querySelector('[data-error]')).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('each setting kind renders its control', () => {
      renderRow(SWITCH);
      expect(screen.getByRole('switch', { name: 'Demo module' })).toBeChecked();
      expect(screen.getByText('Look for the demo findings.')).toBeInTheDocument();

      renderRow(NUMBER);
      expect(screen.getByRole('spinbutton', { name: 'Stale after' })).toHaveValue(30);
      expect(screen.getByText('From 1 to 365 days')).toBeInTheDocument();

      renderRow(FEW_CHOICES);
      expect(screen.getByRole('radiogroup', { name: 'Remove to' })).toBeInTheDocument();
      expect(screen.getByRole('radio', { name: 'Trash' })).toBeChecked();

      renderRow(MANY_CHOICES);
      expect(screen.getByRole('combobox', { name: 'Remind me' })).toHaveValue('week');

      renderRow(FOLDERS, { value: { kind: 'folders', value: ['~/Projects', '~/Work'] } });
      const folders = screen.getByRole('list', { name: 'Project folders' });
      expect(within(folders).getAllByRole('listitem')).toHaveLength(2);
      expect(within(folders).getByText('~/Projects', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('a switch change calls onChange with the new value', () => {
      const onChange = vi.fn();
      renderRow(SWITCH, { onChange });

      fireEvent.click(screen.getByRole('switch', { name: 'Demo module' }));

      expect(onChange).toHaveBeenCalledExactlyOnceWith({ kind: 'switch', value: false });
    });

    it('a number inside its limits is sent as typed', () => {
      const onChange = vi.fn();
      renderRow(NUMBER, { onChange });

      fireEvent.change(screen.getByRole('spinbutton', { name: 'Stale after' }), { target: { value: '90' } });

      expect(onChange).toHaveBeenCalledExactlyOnceWith({ kind: 'number', value: 90 });
    });

    it("the slider shows the core's bounds", () => {
      renderRow(NUMBER, { value: { kind: 'number', value: 90 } });
      const slider = screen.getByRole('slider', { name: 'Stale after, days' });

      expect(slider).toHaveAttribute('min', '1');
      expect(slider).toHaveAttribute('max', '365');
      expect(slider).toHaveValue('90');
      expect(screen.getByText('days')).toBeInTheDocument();
    });

    it('moving the slider sends one value on release and moves the field as it goes', () => {
      const onChange = vi.fn();
      renderRow(NUMBER, { onChange });
      const slider = screen.getByRole('slider', { name: 'Stale after, days' });

      fireEvent.input(slider, { target: { value: '40' } });
      fireEvent.input(slider, { target: { value: '45' } });

      expect(onChange).not.toHaveBeenCalled();
      expect(screen.getByRole('spinbutton', { name: 'Stale after' })).toHaveValue(45);

      fireEvent.pointerUp(slider);

      expect(onChange).toHaveBeenCalledExactlyOnceWith({ kind: 'number', value: 45 });
    });

    it('a released slider stays where it was let go until the stored value comes back', () => {
      const onChange = vi.fn();
      const { rerender } = renderRow(NUMBER, { onChange, value: { kind: 'number', value: 90 } });
      const slider = screen.getByRole('slider', { name: 'Stale after, days' });
      const row = (value: number): JSX.Element => (
        <SettingsRow
          descriptor={NUMBER}
          onChange={onChange}
          problem={null}
          readOnly={false}
          value={{ kind: 'number', value }}
        />
      );

      fireEvent.input(slider, { target: { value: '40' } });
      fireEvent.pointerUp(slider);

      expect(onChange).toHaveBeenCalledExactlyOnceWith({ kind: 'number', value: 40 });
      expect(slider).toHaveValue('40');
      expect(screen.getByRole('spinbutton', { name: 'Stale after' })).toHaveValue(40);

      rerender(row(40));

      expect(slider).toHaveValue('40');

      rerender(row(60));

      expect(slider).toHaveValue('60');
    });

    it('typing a valid number moves the slider', () => {
      const { rerender } = renderRow(NUMBER);

      fireEvent.change(screen.getByRole('spinbutton', { name: 'Stale after' }), { target: { value: '90' } });
      rerender(
        <SettingsRow
          descriptor={NUMBER}
          onChange={vi.fn()}
          problem={null}
          readOnly={false}
          value={{ kind: 'number', value: 90 }}
        />,
      );

      expect(screen.getByRole('slider', { name: 'Stale after, days' })).toHaveValue('90');
    });

    it('a number without a unit names only its limits', () => {
      renderRow({ ...NUMBER, kind: { kind: 'number', max: 10, min: 0, unit: null } });

      expect(screen.getByText('From 0 to 10')).toBeInTheDocument();
    });

    it('a number in bytes names its unit', () => {
      renderRow({ ...NUMBER, kind: { kind: 'number', max: 1000, min: 1, unit: 'bytes' } });

      expect(screen.getByText('From 1 to 1,000 bytes')).toBeInTheDocument();
    });

    it('a number shown again from the outside replaces what was typed', () => {
      const { rerender } = renderRow(NUMBER);
      const field = screen.getByRole('spinbutton', { name: 'Stale after' });

      fireEvent.change(field, { target: { value: '400' } });
      rerender(
        <SettingsRow
          descriptor={NUMBER}
          onChange={vi.fn()}
          problem={null}
          readOnly={false}
          value={{ kind: 'number', value: 60 }}
        />,
      );

      expect(field).toHaveValue(60);
      expect(field).not.toHaveAttribute('aria-invalid');
    });

    it('a choice change calls onChange with the chosen option', () => {
      const onChange = vi.fn();
      renderRow(FEW_CHOICES, { onChange });

      fireEvent.click(screen.getByRole('radio', { name: 'Delete permanently' }));

      expect(onChange).toHaveBeenCalledExactlyOnceWith({ kind: 'choice', value: 'permanent' });
    });

    it('a choice from a long list calls onChange from the select', () => {
      const onChange = vi.fn();
      renderRow(MANY_CHOICES, { onChange });

      fireEvent.change(screen.getByRole('combobox', { name: 'Remind me' }), { target: { value: 'month' } });

      expect(onChange).toHaveBeenCalledExactlyOnceWith({ kind: 'choice', value: 'month' });
    });

    it("a setting's problem shows as its error", () => {
      renderRow(NUMBER, { problem: { key: 'demo.min-age', kind: 'out-of-range', max: 365, min: 1 } });

      expect(screen.getByRole('spinbutton', { name: 'Stale after' })).toHaveAccessibleDescription(
        'The saved value was not between 1 and 365, so the default is used. From 1 to 365 days',
      );
    });

    it('every kind of problem has its words', () => {
      const problems: readonly (readonly [SettingDescriptor, SettingProblem, string])[] = [
        [SWITCH, { key: 'demo.enabled', kind: 'wrong-type' }, 'The saved value was of the wrong kind'],
        [MANY_CHOICES, { key: 'demo.every', kind: 'unknown-choice' }, 'The saved choice is not one of the options'],
        [FOLDERS, { folder: '/System', key: 'demo.roots', kind: 'refused-folder' }, '/System cannot be scanned'],
        [SWITCH, { key: 'demo.old', kind: 'unknown-key' }, 'does not know this setting'],
      ];

      for (const [descriptor, problem, words] of problems) {
        const { unmount } = renderRow(descriptor, { problem });

        expect(screen.getByText(new RegExp(words, 'u'))).toBeInTheDocument();
        unmount();
      }
    });

    it('a value of another kind shows the default', () => {
      renderRow(SWITCH, { value: { kind: 'number', value: 3 } });

      expect(screen.getByRole('switch', { name: 'Demo module' })).toBeChecked();
    });
  });
});
