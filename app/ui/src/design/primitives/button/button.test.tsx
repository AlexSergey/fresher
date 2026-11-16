import { fireEvent, render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { createRef } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { Trash2 } from '../../icons/icons';
import { Button } from './button';

const css = readFileSync(path.join(import.meta.dirname, 'button.module.css'), 'utf8');
const motionCss = readFileSync(path.join(import.meta.dirname, '../../tokens/motion.css'), 'utf8');

// The declarations of the rule with exactly this selector.
const ruleOf = (selector: string): string => {
  const start = css.indexOf(`${selector} {`);

  return start === -1 ? '' : css.slice(start, css.indexOf('}', start));
};

describe('Button', () => {
  describe('negative cases', () => {
    it('does not fire when disabled', () => {
      const onClick = vi.fn();
      render(
        <Button disabled onClick={onClick}>
          Scan
        </Button>,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(onClick).not.toHaveBeenCalled();
    });

    it('does not fire when busy', () => {
      const onClick = vi.fn();
      render(
        <Button busy onClick={onClick} type="submit">
          Scan
        </Button>,
      );

      const button = screen.getByRole('button', { name: 'Scan' });
      const click = new MouseEvent('click', { bubbles: true, cancelable: true });
      fireEvent(button, click);

      expect(onClick).not.toHaveBeenCalled();
      expect(click.defaultPrevented).toBe(true);
      expect(button).toHaveAttribute('aria-disabled', 'true');
    });

    it('is not a submit button by default', () => {
      render(<Button>Scan</Button>);

      expect(screen.getByRole('button', { name: 'Scan' })).toHaveAttribute('type', 'button');
    });

    it('is neither busy nor disabled for assistive technology at rest', () => {
      render(<Button>Scan</Button>);

      const button = screen.getByRole('button', { name: 'Scan' });

      expect(button).not.toHaveAttribute('aria-busy');
      expect(button).not.toHaveAttribute('aria-disabled');
    });

    it('the danger variant has no glow', () => {
      const glowing = [...css.matchAll(/([^{}]+)\{[^}]*--glow-/gu)].map(([, selector]) => (selector ?? '').trim());

      expect(glowing).toStrictEqual([".button[data-variant='primary']::after"]);
      expect(css).not.toMatch(/data-variant='danger'\][^{]*::after/u);
    });
  });

  describe('positive cases', () => {
    it('has its label as its accessible name', () => {
      render(
        <Button icon={Trash2} variant="primary">
          Move to Trash
        </Button>,
      );

      const button = screen.getByRole('button', { name: 'Move to Trash' });

      expect(button).toHaveAttribute('data-variant', 'primary');
      expect(button).toHaveAttribute('data-size', 'medium');
    });

    it('passes its ref to the native button, so a dialog can focus it', () => {
      const ref = createRef<HTMLButtonElement>();
      render(<Button ref={ref}>Cancel</Button>);

      expect(ref.current).toBe(screen.getByRole('button', { name: 'Cancel' }));
    });

    it('fires on click', () => {
      const onClick = vi.fn();
      render(<Button onClick={onClick}>Scan</Button>);

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(onClick).toHaveBeenCalledOnce();
    });

    it('fires without a handler too', () => {
      render(<Button type="submit">Scan</Button>);

      fireEvent.click(screen.getByRole('button', { name: 'Scan' }));

      expect(screen.getByRole('button', { name: 'Scan' })).toHaveAttribute('type', 'submit');
    });

    it('shows the spinner and keeps its label when busy', () => {
      const { container } = render(
        <Button busy icon={Trash2} size="small">
          Move to Trash
        </Button>,
      );

      const button = screen.getByRole('button', { name: 'Move to Trash' });

      expect(button).toHaveAttribute('aria-busy', 'true');
      // eslint-disable-next-line testing-library/no-container, testing-library/no-node-access -- the spinner and the icon are hidden decoration
      const drawn = container.querySelectorAll('svg');

      expect(drawn).toHaveLength(1);
      expect(drawn[0]).toHaveAttribute('width', '12');
    });

    it('the primary glow fades in on hover', () => {
      const glow = ruleOf(".button[data-variant='primary']::after");

      expect(glow).toMatch(/box-shadow: var\(--glow-accent\);/u);
      expect(glow).toMatch(/opacity: 0;/u);
      expect(glow).toMatch(/transition: opacity var\(--duration-fast\) var\(--ease-standard\);/u);
      expect(css).toMatch(/\[data-variant='primary'\]\[data-hover\]::after \{\s*opacity: 1;/u);
      // Motion off makes it instant.
      expect(motionCss).toMatch(/:root\[data-motion='off'\] \{[^}]*--duration-fast: 0ms;/u);
    });

    it('presses by transform', () => {
      expect(css).toMatch(/transform var\(--duration-press\) var\(--ease-standard\)/u);
      expect(css).toMatch(/transform: scale\(var\(--scale-press\)\);/u);
    });
  });
});
