// The licence notices the app ships (plan 013/03/01): the pure part. `scripts/notices.ts` reads the files and runs
// cargo-about; this module turns what they hold into the notices and the problems that stop them.

// One licence of the JSON of `cargo about generate --format json`, as far as the notices need it.
export type CargoAboutLicence = {
  readonly id: string;
  readonly name: string;
  readonly text: string;
  readonly used_by: readonly { readonly crate: { readonly name: string; readonly version: string } }[];
};

// One licence text and every part of the app that is under it.
export type NoticeLicence = {
  readonly id: string;
  readonly name: string;
  readonly text: string;
  readonly usedBy: readonly string[];
};

export type Notices = {
  readonly licences: readonly NoticeLicence[];
  readonly sources: readonly NoticeSource[];
};

// A crate taken under a licence outside the allowlist (MPL-2.0): where its source is.
export type NoticeSource = {
  readonly licence: string;
  readonly name: string;
  readonly url: string;
  readonly version: string;
};

// An installed npm package: its manifest's licence and its folder's licence files, by name.
export type NpmPackage = {
  readonly files: ReadonlyMap<string, string>;
  readonly licence: unknown;
  readonly name: string;
  readonly version: string;
};

// The licence policy of deny.toml: the allowlist, and the crates taken under one more licence.
export type Policy = {
  readonly allow: readonly string[];
  readonly exceptions: ReadonlyMap<string, readonly string[]>;
};

export type Problem = {
  readonly kind:
    'allowlist-differs' | 'licence-not-allowed' | 'no-licence' | 'no-licence-file' | 'not-installed' | 'stale-notices';
  readonly message: string;
  readonly where: string;
};

// The report of cargo-about, as far as the notices need it.
type CargoAbout = { readonly licenses: readonly CargoAboutLicence[] };

// One part as it is under a licence, before the texts are grouped.
type Entry = {
  readonly id: string;
  readonly name: string;
  readonly part: string;
  readonly text: string;
};

// The full names of the allowed licences, so a Rust and an npm part under the same licence read the same.
const NAMES: Readonly<Record<string, string>> = {
  'Apache-2.0': 'Apache License 2.0',
  'BSD-2-Clause': 'BSD 2-Clause License',
  'BSD-3-Clause': 'BSD 3-Clause License',
  'CC0-1.0': 'Creative Commons Zero v1.0 Universal',
  ISC: 'ISC License',
  MIT: 'MIT License',
  'MPL-2.0': 'Mozilla Public License 2.0',
  'Unicode-3.0': 'Unicode License v3',
  Unlicense: 'The Unlicense',
  Zlib: 'zlib License',
};

const CRATES_IO = 'https://crates.io/crates';

const quoted = (list: string): readonly string[] => [...list.matchAll(/"([^"]+)"/gu)].map((match) => match[1] ?? '');

const sameList = (first: readonly string[], second: readonly string[]): boolean =>
  first.length === second.length && first.toSorted().every((item, index) => item === second.toSorted()[index]);

// deny.toml's `[licenses]`: the `allow` list and the `exceptions`, as taplo formats them.
export const parsePolicy = (denyToml: string): Policy => {
  const start = denyToml.indexOf('[licenses]\n');
  const end = denyToml.indexOf('\n[', start + 1);
  const section = start === -1 ? '' : denyToml.slice(start, end === -1 ? undefined : end);
  const allow = quoted(/^allow\s*=\s*\[([^\]]*)\]/mu.exec(section)?.[1] ?? '');
  const exceptions = new Map(
    [...section.matchAll(/\{\s*crate\s*=\s*"([^"]+)",\s*allow\s*=\s*\[([^\]]*)\]/gu)].map(
      (match) => [match[1] ?? '', quoted(match[2] ?? '')] as const,
    ),
  );

  return { allow, exceptions };
};

// about.toml takes the same licences as deny.toml: its `accepted` list and one `[crate] accepted` per exception.
export const checkAboutConfig = (aboutToml: string, policy: Policy): readonly Problem[] => {
  const accepted = quoted(/^accepted\s*=\s*\[([^\]]*)\]/mu.exec(aboutToml)?.[1] ?? '');
  const crates = new Map(
    [...aboutToml.matchAll(/^\[([\w-]+)\]\n+accepted\s*=\s*\[([^\]]*)\]/gmu)].map(
      (match) => [match[1] ?? '', quoted(match[2] ?? '')] as const,
    ),
  );
  const sameExceptions =
    crates.size === policy.exceptions.size &&
    [...policy.exceptions].every(([crate, allow]) => sameList(crates.get(crate) ?? [], allow));

  return sameList(accepted, policy.allow) && sameExceptions
    ? []
    : [
        {
          kind: 'allowlist-differs',
          message: "about.toml's accepted licences and crates differ from deny.toml's [licenses] allow and exceptions",
          where: 'about.toml',
        },
      ];
};

type Token = '(' | ')' | 'AND' | 'OR' | 'WITH' | { readonly id: string };

const tokenize = (expression: string): readonly Token[] =>
  [...expression.matchAll(/[()]|[^\s()]+/gu)].map(([word]) =>
    word === '(' || word === ')' || word === 'AND' || word === 'OR' || word === 'WITH' ? word : { id: word },
  );

// The licences of the first alternative of an SPDX expression the allowlist satisfies, in order; null when none
// does. `WITH` an exception is never satisfied: it is asked about first, as any licence outside the list.
export const chooseLicences = (expression: string, allow: readonly string[]): null | readonly string[] => {
  const tokens = tokenize(expression);
  const state: { at: number; wellFormed: boolean } = { at: 0, wellFormed: true };

  const factor = (): null | readonly string[] => {
    const token = tokens[state.at];
    state.at += 1;
    if (token === '(') {
      const inner = either();
      if (tokens[state.at] === ')') {
        state.at += 1;
      } else {
        state.wellFormed = false;
      }

      return inner;
    }
    if (token === undefined || typeof token === 'string') {
      state.wellFormed = false;

      return null;
    }
    if (tokens[state.at] === 'WITH') {
      state.at += 2;

      return null;
    }

    return allow.includes(token.id) ? [token.id] : null;
  };

  const both = (): null | readonly string[] => {
    let chosen = factor();
    while (tokens[state.at] === 'AND') {
      state.at += 1;
      const next = factor();
      chosen = chosen === null || next === null ? null : [...chosen, ...next];
    }

    return chosen;
  };

  const either = (): null | readonly string[] => {
    let chosen = both();
    while (tokens[state.at] === 'OR') {
      state.at += 1;
      const next = both();
      chosen ??= next;
    }

    return chosen;
  };

  const chosen = either();

  return state.wellFormed && state.at === tokens.length ? chosen : null;
};

// Line ends as LF, no trailing spaces, no blank lines around: the same text reads the same from any package.
const normalize = (text: string): string =>
  text
    .replaceAll('\r\n', '\n')
    .split('\n')
    .map((line) => line.trimEnd())
    .join('\n')
    .replace(/^\n+/u, '')
    .replace(/\n+$/u, '');

const nameOf = (id: string, fallback: string): string => NAMES[id] ?? fallback;

const compare = (first: string, second: string): number => {
  if (first === second) {
    return 0;
  }

  return first < second ? -1 : 1;
};

// The crates of cargo-about's report under their licence texts; a licence outside the allowlist, for a crate not
// among deny.toml's exceptions, is a problem (cargo-about's `--fail` and cargo-deny refuse it too).
export const rustEntries = (
  report: CargoAbout,
  policy: Policy,
): { readonly entries: readonly Entry[]; readonly problems: readonly Problem[]; readonly sources: NoticeSource[] } => {
  const entries: Entry[] = [];
  const problems: Problem[] = [];
  const sources: NoticeSource[] = [];

  for (const licence of report.licenses) {
    for (const { crate } of licence.used_by) {
      const part = `${crate.name} ${crate.version}`;
      if (policy.allow.includes(licence.id)) {
        entries.push({ id: licence.id, name: nameOf(licence.id, licence.name), part, text: licence.text });
      } else if (policy.exceptions.get(crate.name)?.includes(licence.id) === true) {
        entries.push({ id: licence.id, name: nameOf(licence.id, licence.name), part, text: licence.text });
        sources.push({
          licence: licence.id,
          name: crate.name,
          url: `${CRATES_IO}/${crate.name}/${crate.version}`,
          version: crate.version,
        });
      } else {
        problems.push({
          kind: 'licence-not-allowed',
          message: `${licence.id} is outside the allowlist of deny.toml`,
          where: `crate ${part}`,
        });
      }
    }
  }

  return { entries, problems, sources };
};

// A package's licence file for one licence: the only one it has, or the one whose name says the licence.
const fileFor = (files: ReadonlyMap<string, string>, id: string): string | undefined => {
  const named = [...files.keys()].filter((file) => !file.toLowerCase().endsWith('.spdx')).toSorted(compare);
  if (named.length === 1) {
    return files.get(named[0] ?? '');
  }
  const key = (id.split(/[-.]/u)[0] ?? id).toUpperCase();
  const match = named.find((file) =>
    file
      .toUpperCase()
      .replace(/^LICEN[CS]E/u, '')
      .includes(key),
  );

  return match === undefined ? undefined : files.get(match);
};

// The npm production packages under their licence texts: each licence must be in the allowlist (deny.toml's
// exceptions are crates of Tauri, not npm packages) and its text in the package's folder.
export const npmEntries = (
  packages: readonly NpmPackage[],
  policy: Policy,
): { readonly entries: readonly Entry[]; readonly problems: readonly Problem[] } => {
  const entries: Entry[] = [];
  const problems: Problem[] = [];

  for (const npm of packages) {
    const part = `${npm.name} ${npm.version}`;
    const where = `npm ${part}`;
    if (typeof npm.licence !== 'string' || npm.licence.trim() === '') {
      problems.push({ kind: 'no-licence', message: 'its package.json declares no SPDX licence', where });
      continue;
    }
    const chosen = chooseLicences(npm.licence, policy.allow);
    if (chosen === null) {
      problems.push({
        kind: 'licence-not-allowed',
        message: `${npm.licence} is outside the allowlist of deny.toml`,
        where,
      });
      continue;
    }
    for (const id of chosen) {
      const text = fileFor(npm.files, id);
      if (text === undefined) {
        problems.push({ kind: 'no-licence-file', message: `no licence file for ${id} in its folder`, where });
      } else {
        entries.push({ id, name: nameOf(id, id), part, text });
      }
    }
  }

  return { entries, problems };
};

// The entries grouped by licence and text, each group's parts sorted and unique; the groups by name, then by
// their first part. The sources by name and version.
export const buildNotices = (entries: readonly Entry[], sources: readonly NoticeSource[]): Notices => {
  const groups = new Map<string, { id: string; name: string; parts: Set<string>; text: string }>();

  for (const entry of entries) {
    const text = normalize(entry.text);
    const key = `${entry.id}\n${text}`;
    const group = groups.get(key) ?? { id: entry.id, name: entry.name, parts: new Set<string>(), text };
    group.parts.add(entry.part);
    groups.set(key, group);
  }

  const licences = [...groups.values()]
    .map(({ id, name, parts, text }) => ({ id, name, text, usedBy: [...parts].toSorted(compare) }))
    .toSorted(
      (first, second) => compare(first.name, second.name) || compare(first.usedBy[0] ?? '', second.usedBy[0] ?? ''),
    );
  const unique = new Map(sources.map((source) => [`${source.name} ${source.version}`, source] as const));

  return {
    licences,
    sources: [...unique.values()].toSorted(
      (first, second) => compare(first.name, second.name) || compare(first.version, second.version),
    ),
  };
};
