import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import {
  ADMIN_EXTERNAL_FINDING,
  ADMIN_FINDING,
  EXTERNAL_FINDING,
  FINDING_DICTIONARY,
  HIGH_FINDING,
  LOCKED_FINDING,
  MEDIUM_FINDING,
  PARTIAL_FINDING,
  PROTECTED_FINDING,
  SAFE_FINDING,
} from '../../test/finding-fixtures';
import { FindingRow } from './finding-row';

const inList = (row: React.JSX.Element): void => {
  render(<div role="list">{row}</div>);
};

beforeEach(() => {
  setCoreDictionary(FINDING_DICTIONARY);
});

describe('FindingRow', () => {
  describe('negative cases', () => {
    it('a row whose page is loading is a skeleton, not an empty row', () => {
      inList(<FindingRow finding={undefined} onToggle={vi.fn()} selected={false} tabIndex={0} />);

      const row = screen.getByRole('listitem', { name: 'Loading…' });

      expect(row).toHaveAttribute('tabindex', '0');
      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-node-access -- the skeleton lines are hidden decoration
      expect(row.querySelectorAll('[aria-hidden="true"]').length).toBeGreaterThan(0);
    });

    it("a finding's level is never colour alone", () => {
      inList(<FindingRow finding={MEDIUM_FINDING} onToggle={vi.fn()} selected={false} />);

      const row = screen.getByRole('listitem');

      expect(row).toHaveTextContent('Medium');
      // eslint-disable-next-line testing-library/no-node-access -- the badge's icon is decoration beside its word
      expect(row.querySelector('[data-level="medium"] svg')).not.toBeNull();
    });

    it('its checkbox adds no tab stop of its own: the row is the tab stop', () => {
      inList(<FindingRow finding={SAFE_FINDING} onToggle={vi.fn()} selected={false} tabIndex={0} />);

      expect(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' })).toHaveAttribute('tabindex', '-1');
    });

    it('a selected row below High keeps the accent edge', () => {
      inList(<FindingRow finding={SAFE_FINDING} onToggle={vi.fn()} selected />);

      const row = screen.getByRole('listitem');

      expect(row).toHaveAttribute('data-selected');
      expect(row).not.toHaveAttribute('data-level');
    });

    it('an item that needs an administrator has no checkbox', () => {
      inList(<FindingRow finding={ADMIN_FINDING} onReveal={vi.fn()} onToggle={vi.fn()} selected />);

      const row = screen.getByRole('listitem', {
        name: 'Example: shared-cache, Safe, 3.1 GB, Needs an administrator',
      });

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
      expect(row).not.toHaveAttribute('data-selected');
      expect(row).toHaveTextContent('Safe');
      expect(row).toHaveTextContent('3.1 GB');
    });

    it('a protected item offers no instruction and no Finder button', () => {
      inList(<FindingRow finding={PROTECTED_FINDING} onReveal={vi.fn()} onToggle={vi.fn()} selected={false} />);

      fireEvent.focusIn(screen.getByText('Protected by macOS'));

      expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Show in Finder' })).not.toBeInTheDocument();
      expect(screen.getByRole('tooltip')).toHaveTextContent(
        'macOS protects this item to keep the system working. Fresher shows it only so you know what takes the space.',
      );
      expect(screen.getByRole('tooltip')).not.toHaveTextContent('Finder');
    });

    it('a tool object that needs an administrator has no Finder button: it has no place in Finder', () => {
      inList(<FindingRow finding={ADMIN_EXTERNAL_FINDING} onReveal={vi.fn()} onToggle={vi.fn()} selected={false} />);

      expect(screen.getByText('Needs an administrator')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Show in Finder' })).not.toBeInTheDocument();
    });

    it('a row without a reveal handler offers no Finder button', () => {
      inList(<FindingRow finding={LOCKED_FINDING} onToggle={vi.fn()} selected={false} />);

      expect(screen.getByText('Locked')).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Show in Finder' })).not.toBeInTheDocument();
    });

    it('a removable finding shows no removability', () => {
      inList(<FindingRow finding={SAFE_FINDING} onReveal={vi.fn()} onToggle={vi.fn()} selected={false} />);

      expect(screen.queryByText('Can be removed')).not.toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Show in Finder' })).not.toBeInTheDocument();
    });

    it('a partial size is read as a lower bound', () => {
      inList(<FindingRow finding={PARTIAL_FINDING} onToggle={vi.fn()} selected={false} />);

      expect(screen.getByRole('listitem')).toHaveAccessibleName('Example: mail-downloads, Low, at least 640 MB');
    });
  });

  describe('positive cases', () => {
    it('reads title, level and size, and the reason as its description', () => {
      inList(<FindingRow finding={SAFE_FINDING} onToggle={vi.fn()} selected={false} />);

      const row = screen.getByRole('listitem', { name: 'Example: editor-cache, Safe, 1.5 GB' });

      expect(row).toHaveAccessibleDescription('Example data from the demo module: it does not exist on your disk.');
      expect(screen.getByText('/demo/editor-cache', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('the checkbox toggles the finding', () => {
      const onToggle = vi.fn();
      inList(<FindingRow finding={SAFE_FINDING} onToggle={onToggle} selected={false} />);

      fireEvent.click(screen.getByRole('checkbox', { name: 'Select Example: editor-cache' }));

      expect(onToggle).toHaveBeenCalledWith(SAFE_FINDING);
    });

    it('a selected High row lights its edge in High', () => {
      inList(<FindingRow finding={HIGH_FINDING} onToggle={vi.fn()} selected />);

      expect(screen.getByRole('checkbox')).toBeChecked();
      expect(screen.getByRole('listitem')).toHaveAttribute('data-level', 'high');
    });

    it('the instruction is shown on demand', () => {
      inList(<FindingRow finding={ADMIN_FINDING} onReveal={vi.fn()} onToggle={vi.fn()} selected={false} />);
      const how =
        'Only an administrator can remove this. Show it in Finder and drag it to the Trash; macOS will ask for an administrator password.';

      expect(screen.queryByRole('tooltip')).not.toBeInTheDocument();

      const badge = screen.getByText('Needs an administrator');
      fireEvent.focusIn(badge);

      expect(screen.getByRole('tooltip')).toHaveTextContent(how);
      expect(badge).toHaveAccessibleDescription(how);
    });

    it('show in Finder reveals the finding', () => {
      const onReveal = vi.fn();
      inList(<FindingRow finding={LOCKED_FINDING} onReveal={onReveal} onToggle={vi.fn()} selected={false} />);

      fireEvent.click(screen.getByRole('button', { name: 'Show in Finder' }));

      expect(onReveal).toHaveBeenCalledWith(LOCKED_FINDING);
    });

    it('an external subject shows its tool and id', () => {
      inList(<FindingRow finding={EXTERNAL_FINDING} onToggle={vi.fn()} selected={false} />);

      expect(screen.getByText('docker: sha256:4f1c9a')).toBeInTheDocument();
    });

    it('takes the place the virtual list gives it', () => {
      inList(
        <FindingRow
          aria-posinset={3}
          aria-setsize={40}
          finding={SAFE_FINDING}
          onToggle={vi.fn()}
          selected={false}
          tabIndex={-1}
        />,
      );

      const row = screen.getByRole('listitem');

      expect(row).toHaveAttribute('aria-posinset', '3');
      expect(row).toHaveAttribute('aria-setsize', '40');
      expect(row).toHaveAttribute('tabindex', '-1');
    });
  });
});
