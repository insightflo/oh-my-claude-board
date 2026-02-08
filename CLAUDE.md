# OhMyProject Rules

## Reference & Logic
- This project leverages `cokacdir-main` as a reference for directory management and content handling.
- **CRITICAL**: When the user or instructions refer to "directory management", "directory structure", or "reference tool", you **MUST** use `qmd` to search:
  - **Primary Reference**: `ref-cokacdir-main`

- **Reference Commands**:
  - `qmd query -c ref-cokacdir-main "search term"`
- Do not attempt to explore `/Users/kwak/Projects/ref/cokacdir-main` using `ls`, `grep`, or `find` unless `qmd` fails to provide enough context.

## Token Efficiency
- By using `qmd`, you ensure 90%+ token savings when researching reference materials.
- Prefer `qmd get` or `qmd multi_get` for specific code examples discovered during search.
