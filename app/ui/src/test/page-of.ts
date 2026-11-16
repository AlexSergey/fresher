import type { Finding, FindingsPage, FindingsQuery } from '../ipc/bindings.generated';

// `get_findings` as the engine answers it, over a list of findings in the order the scan found them: the matching
// findings of the module and category, in the query's order (`largest`: allocated size, largest first, ties by id),
// from `offset`, at most `limit`, with how many match in all. Shared by the unit tests and the e2e mock.
export const pageOf = (findings: readonly Finding[], query: FindingsQuery): FindingsPage => {
  const matching =
    query.module === null && query.category === null
      ? findings
      : findings.filter(
          (finding) =>
            (query.module === null || finding.module === query.module) &&
            (query.category === null || finding.category === query.category),
        );
  const ordered =
    query.order === 'largest'
      ? [...matching].sort((a, b) => b.sizes.allocated - a.sizes.allocated || a.id - b.id)
      : matching;

  return {
    findings: ordered.slice(query.offset, query.offset + query.limit),
    offset: query.offset,
    total: matching.length,
  };
};
