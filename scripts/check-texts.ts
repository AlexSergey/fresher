import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

// The text check (docs/writing.md): the rules about the app's texts that a script can hold, over every English
// dictionary. The module contract (crates/testing/src/contract.rs) holds the rest.

export type Dictionary = Readonly<Record<string, string>>;

export type RuleId =
  'banned' | 'category-parts' | 'delete' | 'description' | 'exclamation' | 'form' | 'length' | 'sentence' | 'trash';

export type Violation = {
  readonly file: string;
  readonly key: string;
  readonly message: string;
  readonly rule: RuleId;
};

// The longest text, in characters, by the key's last segment; any other segment gets the `other` limit.
export const LIMITS: Readonly<Record<string, number>> = {
  'comes-back': 220,
  description: 200,
  'if-removed': 240,
  label: 48,
  'made-by': 160,
  name: 48,
  other: 240,
  title: 64,
  what: 220,
};

export const BANNED: readonly string[] = [
  'junk',
  'garbage',
  'simply',
  'easily',
  'please',
  'oops',
  'sorry',
  'click here',
  'at risk',
  'you must',
  'directory',
];

// Key segments whose text is a sentence (or a lead-in to a list) and ends so.
export const SENTENCE_KINDS: readonly string[] = [
  'description',
  'what',
  'made-by',
  'if-removed',
  'comes-back',
  'left-out',
  'skip',
  'failed',
];

// Key -> reason. A text exempt from every rule; a new entry is a reviewed change.
export const EXEMPT: ReadonlyMap<string, string> = new Map<string, string>([
  ['ui.confirm.left-out', 'a heading over a list of items, not a sentence'],
  ['ui.finding-list.failed.title', 'a heading of the error state, not a sentence'],
  ['ui.scan-hero.failed', 'a heading of the error state, not a sentence'],
  ['ui.scan-progress.module.failed', 'a one-word status of a module chip'],
  ['ui.screen.category.comes-back', 'the label of a part of the category screen, not a category text'],
  ['ui.screen.category.if-removed', 'the label of a part of the category screen, not a category text'],
  ['ui.screen.category.made-by', 'the label of a part of the category screen, not a category text'],
  ['ui.screen.category.what', 'the label of a part of the category screen, not a category text'],
  ['ui.screen.scan.left-out.title', 'a heading of a section, not a sentence'],
]);

const PLURAL_SEGMENTS: ReadonlySet<string> = new Set(['one', 'other']);
const PART_SEGMENTS: readonly string[] = ['what', 'made-by', 'if-removed', 'comes-back'];
const DELETE_SEGMENTS: ReadonlySet<string> = new Set(['delete', 'deleted', 'deleting', 'permanent']);
const SENTENCE_END = /[.…:]$/;

const escape = (word: string): string => word.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const BANNED_PATTERNS: readonly (readonly [string, RegExp])[] = BANNED.map((word) => [
  word,
  new RegExp(`\\b${escape(word)}\\b`, 'i'),
]);

const segmentsOf = (key: string): string[] => key.split('.');

// The key's kind: its last segment, or the one before it for a plural form.
const kindOf = (segments: readonly string[]): string => {
  const last = segments[segments.length - 1] ?? '';

  return PLURAL_SEGMENTS.has(last) && segments.length > 1 ? (segments[segments.length - 2] ?? last) : last;
};

const prefixOf = (key: string): string => key.slice(0, Math.max(key.lastIndexOf('.'), 0));

type Context = {
  readonly dictionary: Dictionary;
  readonly key: string;
  readonly kind: string;
  readonly segments: readonly string[];
  readonly text: string;
};

type Finding = { readonly message: string; readonly rule: RuleId };

const has = (dictionary: Dictionary, key: string): boolean => Object.hasOwn(dictionary, key);

const checkLength = ({ kind, text }: Context): Finding[] => {
  const limit = (Object.hasOwn(LIMITS, kind) ? LIMITS[kind] : LIMITS.other) ?? 240;

  return text.length > limit
    ? [{ message: `is ${text.length} characters, over the ${limit} of "${kind}"`, rule: 'length' }]
    : [];
};

const checkBanned = ({ text }: Context): Finding[] =>
  BANNED_PATTERNS.filter(([, pattern]) => pattern.test(text)).map(([word]) => ({
    message: `uses the banned word "${word}"`,
    rule: 'banned',
  }));

const checkWording = ({ segments, text }: Context): Finding[] => {
  const found: Finding[] = [];

  if (text.includes('!')) {
    found.push({ message: 'has an exclamation mark', rule: 'exclamation' });
  }
  if (/\bdelet/i.test(text) && !/permanent/i.test(text) && !segments.some((s) => DELETE_SEGMENTS.has(s))) {
    found.push({ message: '"delete" outside a permanent delete', rule: 'delete' });
  }
  if (/\btrash\b/.test(text.replace(/\{[^}]*\}/g, ''))) {
    found.push({ message: 'writes "trash" in lower case; the Trash is a name', rule: 'trash' });
  }

  return found;
};

const checkStructure = ({ dictionary, key, kind }: Context): Finding[] => {
  const found: Finding[] = [];
  const prefix = prefixOf(key);

  if (
    key.endsWith('.label') &&
    !key.startsWith('ui.') &&
    !has(dictionary, `${prefix}.description`) &&
    !has(dictionary, `${prefix}.effect`)
  ) {
    found.push({ message: 'has no description', rule: 'description' });
  }
  if (!key.startsWith('ui.') && PART_SEGMENTS.includes(kind)) {
    const missing = [...PART_SEGMENTS, 'name'].filter((part) => !has(dictionary, `${prefix}.${part}`));

    if (missing.length > 0) {
      found.push({ message: `its category lacks: ${missing.join(', ')}`, rule: 'category-parts' });
    }
  }

  return found;
};

const checkForm = ({ segments, text }: Context): Finding[] => {
  const found: Finding[] = [];

  if (text.includes('  ') || text !== text.trim()) {
    found.push({ message: 'has a double space or a space at an end', rule: 'form' });
  }
  if (segments.some((s) => SENTENCE_KINDS.includes(s)) && !SENTENCE_END.test(text)) {
    found.push({ message: 'does not end with ".", "…" or ":"', rule: 'sentence' });
  }

  return found;
};

const CHECKS: readonly ((context: Context) => Finding[])[] = [
  checkWording,
  checkLength,
  checkBanned,
  checkStructure,
  checkForm,
];

export const checkDictionary = (file: string, dictionary: Dictionary, exempt = EXEMPT): Violation[] =>
  Object.entries(dictionary)
    .filter(([key]) => !exempt.has(key))
    .flatMap(([key, text]) => {
      const segments = segmentsOf(key);
      const context: Context = { dictionary, key, kind: kindOf(segments), segments, text };

      return CHECKS.flatMap((check) => check(context)).map((finding) => ({ file, key, ...finding }));
    });

// Whether the dictionary `path` is there: a module without one is skipped; any other failure to read it is thrown,
// so a dictionary that cannot be read is never passed as checked.
export const hasDictionary = (path: string): boolean => {
  try {
    readFileSync(path);

    return true;
  } catch (error) {
    if (error instanceof Error && 'code' in error && error.code === 'ENOENT') {
      return false;
    }
    throw error;
  }
};

const dictionaryFiles = (): string[] => [
  'crates/core/locales/en.json',
  'app/ui/src/locales/en.json',
  ...readdirSync('modules', { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => join('modules', entry.name, 'locales', 'en.json'))
    .filter(hasDictionary)
    .sort(),
];

if (import.meta.main) {
  const violations = dictionaryFiles().flatMap((file) =>
    checkDictionary(file, JSON.parse(readFileSync(file, 'utf8')) as Dictionary),
  );

  for (const v of violations) {
    process.stderr.write(`${v.file}: ${v.key}: ${v.rule}: ${v.message}\n`);
  }
  process.stderr.write(`check-texts: ${violations.length} violation(s)\n`);

  process.exitCode = violations.length > 0 ? 1 : 0;
}
