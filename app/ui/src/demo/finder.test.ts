import { afterEach, describe, expect, it } from 'vitest';

import type { Target } from '../ipc/bindings.generated';

import { findTarget } from './finder';

const page = (html: string): HTMLElement => {
  const root = document.createElement('div');
  root.innerHTML = html;
  document.body.append(root);

  return root;
};

const button = (name: string): Target => ({ name: { kind: 'exact', text: name }, role: 'button' });

afterEach(() => {
  document.body.replaceChildren();
});

describe('findTarget', () => {
  describe('negative cases', () => {
    it('finds nothing when no element has the role', () => {
      const root = page('<a href="/x">Scan</a>');

      expect(findTarget(root, button('Scan'))).toEqual({ kind: 'none' });
    });

    it('finds nothing when the name differs', () => {
      const root = page('<button>Scan now</button>');

      expect(findTarget(root, button('Scan'))).toEqual({ kind: 'none' });
    });

    it('counts two buttons with the same name', () => {
      const root = page('<button>Scan</button><button>Scan</button>');

      expect(findTarget(root, button('Scan'))).toEqual({ count: 2, kind: 'many' });
    });

    it('counts two buttons with the same name outside the navigation', () => {
      const root = page('<nav><button>Scan</button></nav><button>Scan</button><button>Scan</button>');

      expect(findTarget(root, button('Scan'))).toEqual({ count: 2, kind: 'many' });
    });

    it.each([
      ['hidden', '<div hidden><button>Scan</button></div>'],
      ['aria-hidden', '<div aria-hidden="true"><button>Scan</button></div>'],
      ['inert', '<div inert><button>Scan</button></div>'],
      ['display none', '<div style="display: none"><button>Scan</button></div>'],
      ['visibility hidden', '<button style="visibility: hidden">Scan</button>'],
    ])('does not count a button that is %s', (_, html) => {
      expect(findTarget(page(html), button('Scan'))).toEqual({ kind: 'none' });
    });

    it('does not take a div with the text for a button', () => {
      expect(findTarget(page('<div>Scan</div>'), button('Scan'))).toEqual({ kind: 'none' });
    });

    it('does not take a section without a name for a region', () => {
      const root = page('<section>Storage</section>');

      expect(findTarget(root, { name: { kind: 'exact', text: 'Storage' }, role: 'region' })).toEqual({ kind: 'none' });
    });

    it('does not take a link without an href for a link', () => {
      const root = page('<a>Docs</a>');

      expect(findTarget(root, { name: { kind: 'exact', text: 'Docs' }, role: 'link' })).toEqual({ kind: 'none' });
    });

    it('does not take a text input for a button', () => {
      expect(findTarget(page('<input type="text" value="Scan">'), button('Scan'))).toEqual({ kind: 'none' });
    });
  });

  describe('positive cases', () => {
    it('finds an implicit button by its text, whitespace collapsed', () => {
      const root = page('<button>  Scan \n  now </button>');
      const found = findTarget(root, button('Scan now'));

      expect(found.kind).toBe('one');
      expect(found.kind === 'one' && found.element.tagName).toBe('BUTTON');
    });

    it("takes the page's own button over the same name in the navigation", () => {
      const root = page('<nav aria-label="Screens"><button>Scan</button></nav><main><button>Scan</button></main>');
      const found = findTarget(root, button('Scan'));

      expect(found.kind === 'one' && found.element.closest('main')).not.toBeNull();
    });

    it('still finds a button that only the navigation has', () => {
      expect(findTarget(page('<nav><button>Settings</button></nav>'), button('Settings')).kind).toBe('one');
    });

    it('lets an aria-label win over the text', () => {
      const root = page('<button aria-label="Close">x</button>');

      expect(findTarget(root, button('Close')).kind).toBe('one');
      expect(findTarget(root, button('x'))).toEqual({ kind: 'none' });
    });

    it('names an element by aria-labelledby', () => {
      const root = page('<span id="a">Open</span><span id="b">folder</span><button aria-labelledby="a b">+</button>');

      expect(findTarget(root, button('Open folder')).kind).toBe('one');
    });

    it('matches the start of a name', () => {
      const root = page('<button>Scan 12 items</button>');

      expect(findTarget(root, { name: { kind: 'start', text: 'Scan' }, role: 'button' }).kind).toBe('one');
    });

    it('finds a role="switch" by attribute', () => {
      const root = page('<div role="switch" aria-label="Dark mode"></div>');

      expect(findTarget(root, { name: { kind: 'exact', text: 'Dark mode' }, role: 'switch' }).kind).toBe('one');
    });

    it('finds a checkbox by its label', () => {
      const root = page('<input type="checkbox" id="c"><label for="c">Remember</label>');

      expect(findTarget(root, { name: { kind: 'exact', text: 'Remember' }, role: 'checkbox' }).kind).toBe('one');
    });

    it('finds an input button by its value', () => {
      expect(findTarget(page('<input type="submit" value="Send">'), button('Send')).kind).toBe('one');
    });

    it('finds a heading, a dialog, a row, a link and a named region', () => {
      const root = page(
        '<h2>Title</h2><dialog open>Sure?</dialog><table><tr><td>Cache</td></tr></table>' +
          '<a href="/x">Docs</a><section aria-label="Storage"></section>',
      );
      const named = (role: Target['role'], text: string): Target => ({ name: { kind: 'exact', text }, role });

      expect(findTarget(root, named('heading', 'Title')).kind).toBe('one');
      expect(findTarget(root, named('dialog', 'Sure?')).kind).toBe('one');
      expect(findTarget(root, named('row', 'Cache')).kind).toBe('one');
      expect(findTarget(root, named('link', 'Docs')).kind).toBe('one');
      expect(findTarget(root, named('region', 'Storage')).kind).toBe('one');
    });
  });
});
