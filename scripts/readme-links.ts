import path from 'node:path';

import type { Problem } from './readme-catalogue.ts';

// The README's link, image and claim check (plan 040/01/03). Pure: the file system comes in as functions.

export type LinkKind = 'image' | 'link';

export type LinkProblemKind =
  'dead-anchor' | 'dead-link' | 'image-outside-assets' | 'missing-image' | 'remote-image' | 'unbuilt-claim';

export type Reference = { readonly kind: LinkKind; readonly line: number; readonly target: string };

export const ALLOWED_IMAGE_HOST = 'https://img.shields.io/';

export const UNBUILT_CLAIMS: readonly { readonly pattern: RegExp; readonly reason: string }[] = [
  { pattern: /watch mode/iu, reason: 'watch mode is not built (plan 010)' },
];

const ASSETS_FOLDER = 'assets';
const FENCE = '```';
const UNCHECKED = /^(?:https?:|mailto:)/iu;
const REMOTE = /^https?:\/\//iu;

const TARGET = String.raw`([^)\s]+)(?:\s+"[^"]*")?\)`;
const IMAGE_TEXT = String.raw`!\[[^\]]*\]\([^)]*\)`;
const MARKDOWN_IMAGE = new RegExp(String.raw`!\[[^\]]*\]\(${TARGET}`, 'gu');
const MARKDOWN_LINK = new RegExp(String.raw`(?<!!)\[(?:${IMAGE_TEXT}|[^\][])*\]\(${TARGET}`, 'gu');
const HTML_IMAGE = /<img\s[^>]*?\bsrc=(?:"([^"]*)"|'([^']*)')/giu;
const REFERENCE_DEFINITION = /^ {0,3}\[[^\]]+\]:\s*<?([^\s>]+)>?/u;
const HTML_LINK = /<a\s[^>]*?\bhref=(?:"([^"]*)"|'([^']*)')/giu;

// The lines outside code blocks fenced with three backticks, with their 1-based numbers.
const proseLines = (markdown: string): readonly { readonly line: number; readonly text: string }[] => {
  const lines: { line: number; text: string }[] = [];
  let fenced = false;

  markdown.split('\n').forEach((text, index) => {
    if (text.trim().startsWith(FENCE)) {
      fenced = !fenced;
    } else if (!fenced) {
      lines.push({ line: index + 1, text });
    }
  });

  return lines;
};

export const slugOf = (heading: string): string =>
  heading
    .replace(/^#+/u, '')
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N} _-]/gu, '')
    .replaceAll(' ', '-');

// Repeated headings get `-1`, `-2`, ... the way GitHub numbers them.
export const headingSlugs = (markdown: string): ReadonlySet<string> => {
  const slugs = new Set<string>();

  for (const { text } of proseLines(markdown)) {
    if (/^#{1,6}\s/u.test(text)) {
      const base = slugOf(text);
      let slug = base;
      for (let n = 1; slugs.has(slug); n += 1) {
        slug = `${base}-${n}`;
      }
      slugs.add(slug);
    }
  }

  return slugs;
};

export const findReferences = (markdown: string): readonly Reference[] => {
  const references: Reference[] = [];
  const patterns: readonly (readonly [RegExp, LinkKind])[] = [
    [MARKDOWN_IMAGE, 'image'],
    [MARKDOWN_LINK, 'link'],
    [HTML_IMAGE, 'image'],
    [HTML_LINK, 'link'],
  ];

  for (const { line, text } of proseLines(markdown)) {
    for (const [pattern, kind] of patterns) {
      for (const match of text.matchAll(pattern)) {
        const target = match[1] ?? match[2];
        if (target !== undefined) {
          references.push({ kind, line, target });
        }
      }
    }

    const definition = REFERENCE_DEFINITION.exec(text)?.[1];
    if (definition !== undefined) {
      references.push({ kind: 'link', line, target: definition });
    }
  }

  return references;
};

// A relative target resolved against the file's folder; undefined when it climbs out of the repository.
const resolveTarget = (
  file: string,
  target: string,
): undefined | { readonly anchor: string; readonly repoPath: string } => {
  const hashAt = target.indexOf('#');
  const anchor = hashAt === -1 ? '' : target.slice(hashAt + 1);
  const withoutAnchor = hashAt === -1 ? target : target.slice(0, hashAt);
  const pathPart = withoutAnchor.split('?')[0] ?? '';
  const repoPath = pathPart.startsWith('/')
    ? path.posix.normalize(pathPart.slice(1))
    : path.posix.normalize(path.posix.join(path.posix.dirname(file), pathPart));

  return repoPath === '..' || repoPath.startsWith('../') ? undefined : { anchor, repoPath };
};

const decode = (anchor: string): string => {
  try {
    return decodeURIComponent(anchor).toLowerCase();
  } catch {
    return anchor.toLowerCase();
  }
};

export const checkReferences = (input: {
  readonly exists: (repoPath: string) => boolean;
  readonly file: string;
  readonly markdown: string;
  readonly slugsOf: (repoPath: string) => null | ReadonlySet<string>;
}): readonly Problem[] => {
  const { exists, file, markdown, slugsOf } = input;
  const problems: { readonly line: number; readonly problem: Problem }[] = [];
  const ownSlugs = headingSlugs(markdown);
  const report = (kind: LinkProblemKind, line: number, message: string): void => {
    problems.push({ line, problem: { kind, message, where: `${file}:${line}` } });
  };

  const checkLink = ({ line, target }: Reference): void => {
    // `#` alone is the top of the page.
    if (UNCHECKED.test(target) || target === '' || target === '#') {
      return;
    }
    if (target.startsWith('#')) {
      if (!ownSlugs.has(decode(target.slice(1)))) {
        report('dead-anchor', line, `no heading in this file for ${target}`);
      }

      return;
    }

    const resolved = resolveTarget(file, target);
    if (resolved === undefined || !exists(resolved.repoPath)) {
      report('dead-link', line, `${target} does not point at a file or folder of the repository`);
    } else if (
      resolved.anchor !== '' &&
      resolved.repoPath.endsWith('.md') &&
      !(slugsOf(resolved.repoPath)?.has(decode(resolved.anchor)) ?? false)
    ) {
      report('dead-anchor', line, `no heading in ${resolved.repoPath} for #${resolved.anchor}`);
    }
  };

  const checkImage = ({ line, target }: Reference): void => {
    if (target.startsWith(ALLOWED_IMAGE_HOST)) {
      return;
    }
    if (REMOTE.test(target)) {
      report('remote-image', line, `${target} is a remote image; only ${ALLOWED_IMAGE_HOST} badges are allowed`);

      return;
    }

    const resolved = resolveTarget(file, target);
    if (!resolved?.repoPath.startsWith(`${ASSETS_FOLDER}/`)) {
      report('image-outside-assets', line, `${target} is not under ${ASSETS_FOLDER}/`);
    } else if (!exists(resolved.repoPath)) {
      report('missing-image', line, `${resolved.repoPath} does not exist`);
    }
  };

  for (const reference of findReferences(markdown)) {
    if (reference.kind === 'image') {
      checkImage(reference);
    } else {
      checkLink(reference);
    }
  }

  for (const { line, text } of proseLines(markdown)) {
    for (const { pattern, reason } of UNBUILT_CLAIMS) {
      if (pattern.test(text)) {
        report('unbuilt-claim', line, reason);
      }
    }
  }

  return problems.toSorted((a, b) => a.line - b.line).map(({ problem }) => problem);
};
