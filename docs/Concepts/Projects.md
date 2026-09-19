# Projects

What the new-agent picker offers. `src/core/projects.rs`.

## Where it looks

`$ATRIUM_PROJECTS` if it is set, else `~/projects`, else the current directory.
Overridable so the scan is not hardcoded to one machine's layout.

## What counts as a project

Any directory containing `.git`. A repository is a **leaf**: nothing inside one
is offered separately, so a submodule or a nested checkout does not clutter the
list with near-duplicates of its parent.

## What the scan avoids

- **Depth 3 and no further.** Repositories here sit at `<root>/<group>/<repo>`,
  and stopping early keeps the scan off deep build trees.
- **Dotted directories**, and `node_modules`, `target`, `.cargo`, `.venv`,
  `vendor`.
- **Symlinks**, which are skipped rather than followed, so a link back up the
  tree cannot send the scan round in circles.

Results are sorted, and each project is named after its own directory — the same
name that ends up on the [[Sidebar]] row.

See [[Modals]] for how the list is filtered and ranked.
