import { screen, within } from '@testing-library/react';

// The category cards' headings in a list (a danger zone's, or the grid's own), in order.
export const cardNames = (list = 'Categories'): readonly (null | string)[] =>
  within(screen.getByRole('list', { name: list }))
    .getAllByRole('heading', { level: 3 })
    .map((heading) => heading.textContent);

// The list item of a card, by its name.
export const cardOf = (name: string, list = 'Categories'): HTMLElement =>
  within(screen.getByRole('list', { name: list }))
    .getAllByRole('listitem')
    .find((item) => within(item).queryByRole('heading', { name }) !== null) as HTMLElement;
