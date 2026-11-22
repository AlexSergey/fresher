import type { NameMatch, Role, Target } from '../ipc/bindings.generated';

export type Found =
  | { readonly count: number; readonly kind: 'many' }
  | { readonly element: HTMLElement; readonly kind: 'one' }
  | { readonly kind: 'none' };

const HEADINGS = new Set(['H1', 'H2', 'H3', 'H4', 'H5', 'H6']);
const BUTTON_INPUTS = new Set(['button', 'submit']);

const collapse = (text: null | string): string => (text ?? '').replaceAll(/\s+/gu, ' ').trim();

// The role an element has: the `role` attribute when it names one, else the implicit role of its tag.
const roleOf = (element: HTMLElement): null | string => {
  const explicit = element.getAttribute('role')?.trim().split(/\s+/u)[0];
  if (explicit !== undefined && explicit !== '') {
    return explicit;
  }
  const tag = element.tagName;
  if (tag === 'BUTTON') {
    return 'button';
  }
  if (tag === 'INPUT') {
    const type = element.getAttribute('type') ?? 'text';
    if (type === 'checkbox') {
      return 'checkbox';
    }

    return BUTTON_INPUTS.has(type) ? 'button' : null;
  }
  if (tag === 'A') {
    return element.hasAttribute('href') ? 'link' : null;
  }
  if (HEADINGS.has(tag)) {
    return 'heading';
  }
  if (tag === 'DIALOG') {
    return 'dialog';
  }
  if (tag === 'TR') {
    return 'row';
  }

  return tag === 'SECTION' ? 'region' : null;
};

const labelledBy = (element: HTMLElement): string => {
  const ids = (element.getAttribute('aria-labelledby') ?? '').split(/\s+/u).filter((id) => id !== '');
  const root = element.ownerDocument;

  return collapse(ids.map((id) => root.getElementById(id)?.textContent ?? '').join(' '));
};

export const labelOf = (element: HTMLElement): string => {
  const { id } = element;
  const own = id === '' ? null : element.ownerDocument.querySelector(`label[for="${CSS.escape(id)}"]`);
  const label = own ?? element.closest('label');

  return collapse(label?.textContent ?? null);
};

// The accessible name: aria-label, else aria-labelledby, else an input's label, else the text content. A `<section>`
// is a region only through a name of its own, so its content is not its name.
export const nameOf = (element: HTMLElement): string => {
  const aria = collapse(element.getAttribute('aria-label'));
  if (aria !== '') {
    return aria;
  }
  const referenced = labelledBy(element);
  if (referenced !== '') {
    return referenced;
  }
  if (element.tagName === 'INPUT') {
    const input = element as HTMLInputElement;
    const label = labelOf(element);

    return label === '' && BUTTON_INPUTS.has(input.type) ? collapse(input.value) : label;
  }

  return element.tagName === 'SECTION' && !element.hasAttribute('role') ? '' : collapse(element.textContent);
};

const HIDING = '[hidden], [inert], [aria-hidden="true"]';

const isVisible = (element: HTMLElement): boolean => {
  if (element.closest(HIDING) !== null) {
    return false;
  }
  const view = element.ownerDocument.defaultView;
  for (let node: HTMLElement | null = element; node !== null; node = node.parentElement) {
    const style = view?.getComputedStyle(node);
    if (style?.display === 'none' || style?.visibility === 'hidden') {
      return false;
    }
  }

  return true;
};

const matches = (name: string, wanted: NameMatch): boolean =>
  wanted.kind === 'exact' ? name === wanted.text : name.startsWith(wanted.text);

const NAVIGATION = 'nav, [role="navigation"]';

// The visible elements that are the target: a role and an accessible name, as the e2e tests address them. The sidebar
// repeats the names of screens ("Scan"): when the sidebar and the page both have a match, the page's own wins.
export const findTarget = (root: ParentNode, target: Target): Found => {
  const role: Role = target.role;
  const matching = [...root.querySelectorAll<HTMLElement>('*')].filter(
    (element) => roleOf(element) === role && matches(nameOf(element), target.name) && isVisible(element),
  );
  const outside = matching.filter((element) => element.closest(NAVIGATION) === null);
  const found = outside.length === 0 ? matching : outside;

  if (found.length === 0) {
    return { kind: 'none' };
  }
  const [element] = found;

  return found.length === 1 && element !== undefined ? { element, kind: 'one' } : { count: found.length, kind: 'many' };
};
