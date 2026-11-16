import { render, screen } from '@testing-library/react';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { describe, expect, it } from 'vitest';

import { AppShell } from './app-shell';

const css = readFileSync(path.join(import.meta.dirname, 'app-shell.module.css'), 'utf8');

describe('AppShell', () => {
  describe('negative cases', () => {
    it('renders no light behind the content unless given', () => {
      render(<AppShell sidebar={<nav aria-label="Screens" />}>Content</AppShell>);

      // eslint-disable-next-line testing-library/no-node-access -- the light's layer has no role to query by
      expect(screen.getByRole('main').children).toHaveLength(1);
    });

    it('leaves the sidebar to the native material in the app', () => {
      expect(css).toMatch(
        /\[data-window='vibrant'\] \.sidebar \{\s*background: var\(--surface-sidebar-over-material\);\s*\}/u,
      );
    });

    it('narrows the gutters under 1,000 px, not the sidebar', () => {
      expect(css).toMatch(
        /@container shell \(max-width: 999px\) \{\s*\.main \{\s*--layout-gutter: var\(--size-gutter-narrow\);\s*\}\s*\}/u,
      );
      expect(css).toMatch(/grid-template-columns: var\(--size-sidebar\) minmax\(0, 1fr\);/u);
    });
  });

  describe('positive cases', () => {
    it('puts the sidebar beside the main pane', () => {
      render(<AppShell sidebar={<nav aria-label="Screens" />}>Content</AppShell>);

      expect(screen.getByRole('navigation', { name: 'Screens' })).toBeInTheDocument();
      expect(screen.getByRole('main')).toHaveTextContent('Content');
    });

    it('places the light behind the content', () => {
      render(
        <AppShell ambient={<div data-testid="light" />} sidebar={<nav aria-label="Screens" />}>
          Content
        </AppShell>,
      );
      // eslint-disable-next-line testing-library/no-node-access -- the light's layer has no role to query by
      const [light, content] = screen.getByRole('main').children;

      expect(light).toContainElement(screen.getByTestId('light'));
      expect(light).toHaveAttribute('aria-hidden', 'true');
      expect(content).toHaveTextContent('Content');
    });

    it('paints the fallback sidebar outside the app', () => {
      expect(css).toMatch(/\.sidebar \{[^}]*background: var\(--surface-sidebar\);/u);
    });
  });
});
