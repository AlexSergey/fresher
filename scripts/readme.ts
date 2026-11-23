import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import * as prettier from 'prettier';

import {
  buildBlock,
  parseCatalogue,
  parseLevels,
  type Problem,
  readCategories,
  replaceBlock,
} from './readme-catalogue.ts';
import { checkReferences, headingSlugs } from './readme-links.ts';

// `npm run readme:generate` writes the README's "What it finds" block from docs/categories; `npm run check:readme`
// fails when the README differs from what it would write (plan 040/01/02). The only file-system and Prettier code.

export type Mode = 'check' | 'generate';

const SCRIPT = 'readme';

export const formatMarkdown = async (text: string, filePath: string): Promise<string> => {
  const config = await prettier.resolveConfig(filePath);

  return prettier.format(text, { ...config, filepath: filePath });
};

const readCategoryFiles = (folder: string): ReadonlyMap<string, string> =>
  new Map(
    readdirSync(folder)
      .filter((name) => name.endsWith('.md') && name !== 'README.md')
      .toSorted()
      .map((name) => [name, readFileSync(path.join(folder, name), 'utf8')]),
  );

const GITHUB_FOLDER = '.github';

// Case-exact: macOS file systems are case-insensitive, GitHub is not.
const existsExact = (root: string, repoPath: string): boolean => {
  let folder = root;

  for (const segment of repoPath.split('/').filter((part) => part !== '' && part !== '.')) {
    try {
      if (!readdirSync(folder).includes(segment)) {
        return false;
      }
    } catch {
      return false;
    }
    folder = path.join(folder, segment);
  }

  return existsSync(folder);
};

// README.md and every .github/*.md (the issue templates are YAML), checked against the working tree.
const checkLinks = (root: string, readme: string): readonly Problem[] => {
  const githubFolder = path.join(root, GITHUB_FOLDER);
  const githubFiles = existsSync(githubFolder)
    ? readdirSync(githubFolder)
        .filter((name) => name.endsWith('.md'))
        .toSorted()
        .map((name) => `${GITHUB_FOLDER}/${name}`)
    : [];
  const slugsOf = (repoPath: string): null | ReadonlySet<string> => {
    try {
      return repoPath.endsWith('.md') && existsExact(root, repoPath)
        ? headingSlugs(readFileSync(path.join(root, repoPath), 'utf8'))
        : null;
    } catch {
      return null;
    }
  };

  return ['README.md', ...githubFiles].flatMap((file) =>
    checkReferences({
      exists: (repoPath) => existsExact(root, repoPath),
      file,
      markdown: file === 'README.md' ? readme : readFileSync(path.join(root, file), 'utf8'),
      slugsOf,
    }),
  );
};

export const run = async (mode: Mode, root: string): Promise<readonly Problem[]> => {
  const folder = path.join(root, 'docs', 'categories');
  const readmePath = path.join(root, 'README.md');
  const catalogueReadme = readFileSync(path.join(folder, 'README.md'), 'utf8');

  const { levels, problems: levelProblems } = parseLevels(catalogueReadme);
  const { categories, problems: categoryProblems } = readCategories(
    parseCatalogue(catalogueReadme),
    readCategoryFiles(folder),
  );
  const catalogueProblems = [...levelProblems, ...categoryProblems];

  const readme = readFileSync(readmePath, 'utf8');
  const replaced = replaceBlock(readme, buildBlock(categories, levels));
  if ('problem' in replaced) {
    return [...catalogueProblems, replaced.problem, ...checkLinks(root, readme)];
  }

  // A block built from an inconsistent catalogue is neither written nor compared.
  if (catalogueProblems.length > 0) {
    return [...catalogueProblems, ...checkLinks(root, readme)];
  }

  const formatted = await formatMarkdown(replaced.text, readmePath);
  if (mode === 'generate') {
    writeFileSync(readmePath, formatted);

    return checkLinks(root, formatted);
  }

  const stale: readonly Problem[] =
    formatted === readme
      ? []
      : [
          {
            kind: 'stale-block',
            message:
              'README.md differs from what `npm run readme:generate` writes (the generated block or its formatting)',
            where: 'README.md',
          },
        ];

  return [...stale, ...checkLinks(root, readme)];
};

const parseMode = (argument: string | undefined): Mode | undefined =>
  argument === 'check' || argument === 'generate' ? argument : undefined;

if (import.meta.main) {
  const mode = parseMode(process.argv[2]);

  if (mode === undefined) {
    process.stderr.write(`${SCRIPT}: usage: node scripts/readme.ts <generate | check>\n`);
    process.exitCode = 2;
  } else {
    const problems = await run(mode, path.resolve(import.meta.dirname, '..'));

    for (const problem of problems) {
      process.stderr.write(`${problem.where}: ${problem.kind}: ${problem.message}\n`);
    }

    process.exitCode = problems.length > 0 ? 1 : 0;
  }
}
