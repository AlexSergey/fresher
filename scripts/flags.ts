import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

// The flags guard (plan 043): `flags.toml` holds its shape and every flag is read by a source. The override rules
// repeat app/tauri/build/flags.rs; the release's text ends with the flags the build ships with.

export type Field = readonly string[] | string;

export type Flag = { readonly fields: Readonly<Record<string, Field>>; readonly name: string };

export type Resolved = { readonly about: string; readonly name: string; readonly value: string };

export type Source = { readonly path: string; readonly text: string };

export type Violation = { readonly message: string };

export const KEYS: readonly string[] = ['about', 'type', 'values', 'release', 'development'];
export const ABOUT_LIMIT = 120;

const TABLE = /^\[([\w-]+)\]$/u;
const NAME = /^[a-z][a-z0-9]*$/u;
const TYPE = /^[A-Z][A-Za-z0-9]*$/u;
const STRING_FIELD = /^([a-z]+) = "([^"]*)"$/u;
const LIST_FIELD = /^([a-z]+) = \[((?:"[^"]*"(?:, )?)*)\]$/u;
const KEBAB = /^[a-z][a-z0-9]*(?:-[a-z][a-z0-9]*)*$/u;

const stringsOf = (list: string): string[] => [...list.matchAll(/"([^"]*)"/gu)].map((match) => match[1] ?? '');

// The strict reader: comments, blank lines, `[name]`, `key = "string"` and `key = ["a", "b"]`; any other line is a
// violation naming its number.
export const readFlags = (text: string): { flags: Flag[]; violations: Violation[] } => {
  const flags: { fields: Record<string, Field>; name: string }[] = [];
  const violations: Violation[] = [];
  text.split('\n').forEach((raw, index) => {
    const line = raw.trim();
    const where = `flags.toml:${index + 1}`;
    if (line === '' || line.startsWith('#')) {
      return;
    }
    const table = TABLE.exec(line);
    if (table !== null) {
      flags.push({ fields: {}, name: table[1] ?? '' });

      return;
    }
    const current = flags[flags.length - 1];
    const str = STRING_FIELD.exec(line);
    const list = LIST_FIELD.exec(line);
    const key = str?.[1] ?? list?.[1];
    if (key === undefined) {
      violations.push({ message: `${where}: cannot read "${line}"` });
    } else if (current === undefined) {
      violations.push({ message: `${where}: "${key}" comes before any [flag]` });
    } else if (Object.hasOwn(current.fields, key)) {
      violations.push({ message: `${where}: "${key}" is set twice in [${current.name}]` });
    } else {
      current.fields[key] = str === null ? stringsOf(list?.[2] ?? '') : (str[2] ?? '');
    }
  });

  return { flags, violations };
};

// A source the flags are read from: tests, stories and generated files do not count.
export const isReadingSource = (path: string): boolean =>
  /\.(?:rs|tsx?)$/u.test(path) &&
  !/\.(?:generated|stories|test|spec)\.tsx?$/u.test(path) &&
  !/(?:^|\/)tests?(?:\.rs|\/)/u.test(path);

const stringOf = (flag: Flag, key: string): string | undefined => {
  const field = flag.fields[key];

  return typeof field === 'string' ? field : undefined;
};

const valuesOf = (flag: Flag): readonly string[] => {
  const field = flag.fields.values;

  return typeof field === 'string' || field === undefined ? [] : field;
};

const checkAbout = (flag: Flag): string[] => {
  const about = stringOf(flag, 'about');
  if (about === undefined) {
    return [];
  }
  if (about.trim() === '' || !about.endsWith('.') || about.includes('. ') || about.length > ABOUT_LIMIT) {
    return [`[${flag.name}] about must be one sentence ending in a full stop, at most ${ABOUT_LIMIT} characters`];
  }

  return [];
};

const checkValues = (flag: Flag): string[] => {
  const values = valuesOf(flag);
  const problems: string[] = [];
  if (values.length < 2) {
    problems.push(`[${flag.name}] values needs at least two values`);
  }
  for (const value of values) {
    if (!KEBAB.test(value)) {
      problems.push(`[${flag.name}] value "${value}" is not kebab-case`);
    }
    if (values.indexOf(value) !== values.lastIndexOf(value)) {
      problems.push(`[${flag.name}] value "${value}" is repeated`);
    }
  }
  for (const key of ['release', 'development']) {
    const value = stringOf(flag, key);
    if (value !== undefined && !values.includes(value)) {
      problems.push(`[${flag.name}] ${key} "${value}" is not among its values`);
    }
  }

  return problems;
};

const checkKeys = (flag: Flag): string[] => [
  ...(NAME.test(flag.name) ? [] : [`[${flag.name}] the name must match [a-z][a-z0-9]*`]),
  ...(TYPE.test(stringOf(flag, 'type') ?? 'T') ? [] : [`[${flag.name}] type must be a PascalCase identifier`]),
  ...KEYS.filter((key) => !Object.hasOwn(flag.fields, key)).map((key) => `[${flag.name}] has no "${key}"`),
  ...Object.keys(flag.fields)
    .filter((key) => !KEYS.includes(key))
    .map((key) => `[${flag.name}] has an unknown key "${key}"`),
  ...['about', 'type', 'release', 'development']
    .filter((key) => Object.hasOwn(flag.fields, key) && stringOf(flag, key) === undefined)
    .map((key) => `[${flag.name}] "${key}" must be a string`),
];

const checkRead = (flag: Flag, sources: readonly Source[]): string[] => {
  const reads = new RegExp(`\\b(?:flags|FLAGS)\\.${flag.name}\\b`, 'u');

  return sources.some((source) => isReadingSource(source.path) && reads.test(source.text))
    ? []
    : [`[${flag.name}] is read by no source as flags.${flag.name} or FLAGS.${flag.name}`];
};

export const checkFlags = (flags: readonly Flag[], sources: readonly Source[]): Violation[] =>
  flags
    .flatMap((flag) => [...checkKeys(flag), ...checkAbout(flag), ...checkValues(flag), ...checkRead(flag, sources)])
    .map((message) => ({ message }));

export type Resolution = { readonly errors: string[]; readonly resolved: Resolved[] };

// The build script's override rules: comma-separated `name=value`, spaces trimmed around items, names and values; an
// empty override names none. Refused: an item without `=`, an unknown name, a value outside the flag's values, a name
// twice.
export const resolveRelease = (flags: readonly Flag[], overrides: string | undefined): Resolution => {
  const resolved = flags.map((flag) => ({
    about: stringOf(flag, 'about') ?? '',
    name: flag.name,
    value: stringOf(flag, 'release') ?? '',
  }));
  const errors: string[] = [];
  const named = new Set<string>();
  for (const item of (overrides ?? '')
    .split(',')
    .map((part) => part.trim())
    .filter((part) => part !== '')) {
    const at = item.indexOf('=');
    if (at === -1) {
      errors.push(`FRESHER_FLAGS: \`${item}\` has no \`=\`; write name=value`);
      continue;
    }
    const name = item.slice(0, at).trim();
    const value = item.slice(at + 1).trim();
    const index = flags.findIndex((flag) => flag.name === name);
    const target = resolved[index];
    const flag = flags[index];
    if (target === undefined || flag === undefined) {
      errors.push(`FRESHER_FLAGS: there is no flag \`${name}\``);
    } else if (named.has(name)) {
      errors.push(`FRESHER_FLAGS: \`${name}\` is named twice`);
    } else if (!valuesOf(flag).includes(value)) {
      errors.push(`FRESHER_FLAGS: \`${value}\` is not a value of \`${name}\``);
    } else {
      named.add(name);
      resolved[index] = { ...target, value };
    }
  }

  return { errors, resolved };
};

// The release text's last section: one line per flag, in the file's order. The build script sorts by name instead; the
// order is cosmetic, the generated struct literal does not depend on it.
export const notesSection = (resolved: readonly Resolved[]): string =>
  `## This build\n\n${resolved.map((flag) => `- \`${flag.name}\`: ${flag.value} — ${flag.about}`).join('\n')}\n`;

// The "This build" section for the text of `flags.toml` and the build's override; a file the reader refuses or an
// override it refuses gives problems and no section.
export const buildSection = (text: string, overrides: string | undefined): { problems: string[]; section?: string } => {
  const { flags, violations } = readFlags(text);
  if (violations.length > 0) {
    return { problems: violations.map((violation) => violation.message) };
  }
  const { errors, resolved } = resolveRelease(flags, overrides);

  return errors.length > 0 ? { problems: errors } : { problems: [], section: notesSection(resolved) };
};

const FLAGS_FILE = 'flags.toml';
const SOURCE_ROOTS: readonly string[] = ['app/tauri/src', 'app/ui/src'];

const sourcesOf = (root: string): Source[] =>
  SOURCE_ROOTS.flatMap((dir) =>
    readdirSync(join(root, dir), { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => join(entry.parentPath, entry.name))
      .map((path) => ({ path: relative(root, path), text: '' }))
      .filter((source) => isReadingSource(source.path))
      .map((source) => ({ ...source, text: readFileSync(join(root, source.path), 'utf8') })),
  );

if (import.meta.main) {
  const root = fileURLToPath(new URL('..', import.meta.url));
  const { flags, violations } = readFlags(readFileSync(join(root, FLAGS_FILE), 'utf8'));
  if (process.argv[2] === 'check' && process.argv.length === 3) {
    const found = [...violations, ...(violations.length === 0 ? checkFlags(flags, sourcesOf(root)) : [])];
    for (const violation of found) {
      process.stderr.write(`check-flags: ${violation.message}\n`);
    }
    process.stderr.write(`check-flags: ${found.length} violation(s)\n`);
    process.exitCode = found.length > 0 ? 1 : 0;
  } else {
    for (const violation of violations) {
      process.stderr.write(`check-flags: ${violation.message}\n`);
    }
    process.stderr.write('check-flags: usage: flags.ts check\n');
    process.exitCode = 2;
  }
}
