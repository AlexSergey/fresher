import styles from './pointer.module.css';

export const GLIDE_MS = 600;
const PRESS_MS = 160;
const SVG = 'http://www.w3.org/2000/svg';

export type Pointer = {
  readonly moveTo: (x: number, y: number) => Promise<void>;
  readonly press: () => Promise<void>;
  readonly remove: () => void;
};

const wait = (ms: number): Promise<void> =>
  new Promise((resolve) => {
    setTimeout(resolve, ms);
  });

const place = (element: HTMLElement, x: number, y: number): void => {
  element.style.transform = `translate(${x}px, ${y}px)`;
};

const arrow = (document: Document): SVGSVGElement => {
  const svg = document.createElementNS(SVG, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('class', styles.arrow ?? '');
  const path = document.createElementNS(SVG, 'path');
  path.setAttribute('d', 'M3 2 L3 19 L7.5 14.8 L10.6 21.5 L13.4 20.3 L10.4 13.7 L16.5 13.4 Z');
  svg.append(path);

  return svg;
};

// A fixed arrow that starts at the window's centre. `moveTo` glides it (the tip goes to the point) and resolves when
// the glide ends, or after its time when no transition runs.
export const createPointer = (document: Document): Pointer => {
  const element = document.createElement('div');
  element.className = styles.pointer ?? '';
  element.setAttribute('data-demo-player', '');
  const svg = arrow(document);
  element.append(svg);
  const view = document.defaultView;
  place(element, (view?.innerWidth ?? 0) / 2, (view?.innerHeight ?? 0) / 2);
  document.body.append(element);

  return {
    moveTo: async (x, y): Promise<void> => {
      const ended = new Promise<void>((resolve) => {
        element.addEventListener('transitionend', () => {
          resolve();
        });
      });
      place(element, x, y);
      await Promise.race([ended, wait(GLIDE_MS)]);
    },
    press: async (): Promise<void> => {
      svg.classList.add(styles.pressed ?? '');
      await wait(PRESS_MS / 2);
      svg.classList.remove(styles.pressed ?? '');
      await wait(PRESS_MS / 2);
    },
    remove: (): void => {
      element.remove();
    },
  };
};
