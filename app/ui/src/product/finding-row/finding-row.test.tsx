import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { MenuItem } from '../../design/primitives/menu/menu';

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

const MENU = (onSelect: () => void): readonly MenuItem[] => [{ key: 'ignore', label: 'Ignore this item', onSelect }];

beforeEach(() => {
  setCoreDictionary(FINDING_DICTIONARY);
});

describe('FindingRow', () => {
  describe('negative cases', () => {
    it('has no menu without items', () => {
      inList(<FindingRow finding={SAFE_FINDING} onToggle={vi.fn()} selected={false} tabIndex={0} />);

      expect(screen.queryByRole('button', { name: /^Options for/ })).not.toBeInTheDocument();
    });

    it('a row with an empty menu keeps the menu column and exposes no button', () => {
      const { container } = render(
        <div role="list">
          <FindingRow finding={SAFE_FINDING} menu={[]} onToggle={vi.fn()} selected={false} tabIndex={0} />
        </div>,
      );

      expect(screen.queryByRole('button', { name: /^Options for/ })).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-container, testing-library/no-node-access -- the placeholder is hidden decoration with no role
      expect(container.querySelector('[aria-hidden="true"][class*="menu"]')).toBeInTheDocument();
    });

    it('the menu button of a row that is not the tab stop is out of the tab order', () => {
      inList(
        <FindingRow finding={SAFE_FINDING} menu={MENU(vi.fn())} onToggle={vi.fn()} selected={false} tabIndex={-1} />,
      );

      expect(screen.getByRole('button', { name: 'Options for Example: editor-cache' })).toHaveAttribute(
        'tabindex',
        '-1',
      );
    });

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
    it('a default row shows the full path, whatever the home', () => {
      inList(<FindingRow finding={SAFE_FINDING} home="/demo" onToggle={vi.fn()} selected={false} />);

      expect(screen.getByText('/demo/editor-cache', { selector: '[aria-hidden="true"]' })).toBeInTheDocument();
      expect(screen.queryByText('~/editor-cache')).not.toBeInTheDocument();
    });

    it('a compact row shows no level badge', () => {
      inList(<FindingRow finding={HIGH_FINDING} onToggle={vi.fn()} selected variant="review" />);

      const row = screen.getByRole('listitem');

      expect(row).toHaveAttribute('data-compact', 'true');
      expect(row).not.toHaveAttribute('data-level');
      expect(screen.queryByText('High')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the menu button of the focused row is reached with Tab and named after the finding', () => {
      inList(
        <FindingRow finding={SAFE_FINDING} menu={MENU(vi.fn())} onToggle={vi.fn()} selected={false} tabIndex={0} />,
      );

      expect(screen.getByRole('button', { name: 'Options for Example: editor-cache' })).toHaveAttribute(
        'tabindex',
        '0',
      );
    });

    it('choosing an item calls it once', () => {
      const onSelect = vi.fn();
      inList(
        <FindingRow finding={SAFE_FINDING} menu={MENU(onSelect)} onToggle={vi.fn()} selected={false} tabIndex={0} />,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Options for Example: editor-cache' }));
      fireEvent.click(screen.getByRole('menuitem', { name: 'Ignore this item' }));

      expect(onSelect).toHaveBeenCalledTimes(1);
    });

    it('reads title, level and size, and the reason as its description', () => {
      inList(<FindingRow finding={SAFE_FINDING} onToggle={vi.fn()} selected={false} />);

      const row = screen.getByRole('listitem', { name: 'Example: editor-cache, Safe, 1.5 GB' });

      expect(row).toHaveAccessibleDescription('Made up by the demo; it is not on your disk.');
      expect(screen.getByText('/demo/editor-cache', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('a review row shows its path from ~, its accessible text keeping the full path', () => {
      inList(<FindingRow finding={SAFE_FINDING} home="/demo" onToggle={vi.fn()} selected variant="review" />);

      expect(screen.getByText('~/editor-cache')).toBeInTheDocument();
      expect(screen.getByText('/demo/editor-cache', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('a details row shows no level badge, keeps the level in its name and its path from ~', () => {
      inList(<FindingRow finding={SAFE_FINDING} home="/demo" onToggle={vi.fn()} selected={false} variant="details" />);

      expect(screen.getByRole('listitem', { name: 'Example: editor-cache, Safe, 1.5 GB' })).toBeInTheDocument();
      expect(screen.queryByText('Safe')).toBeNull();
      expect(screen.getByText('~/editor-cache')).toBeInTheDocument();
    });

    it('a compact row still reads the level in its name', () => {
      inList(<FindingRow finding={MEDIUM_FINDING} onToggle={vi.fn()} selected={false} variant="review" />);

      expect(screen.getByRole('listitem', { name: /, Medium, / })).toBeInTheDocument();
      expect(screen.getByRole('checkbox', { name: /^Select / })).not.toBeChecked();
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
        'Only an administrator can remove this. Show it in Finder and drag it to the Trash; macOS asks for the password.';

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
