# devkit-poe-complete

`devkit-complete`: shell completion for `poe` served from Rust (~13 ms per Tab instead of
poe's ~200 ms). `devkit setup-project` adds this package to every project's dev group at
the newest release the installed devkit accepts and runs `devkit-complete install` for the
shells on `PATH`, so nobody types the command. The shims call the `devkit-complete` an
activated venv puts on `PATH`; no global install is needed.

Subcommands: `query` (the per-Tab request, called by the shims), `tasks [DIR]` and `args
<TASK> [DIR]` (retained for shims installed by an older devkit), `script
--powershell|--bash`, `install --powershell --bash [--dry-run]`; global `--no-cache`.

- **Thin shims** - Each shell installs a ~50-line shim that forwards the command line to
  `devkit-complete query` and acts on a directory/file sentinel; all the logic (task
  location, global options, choices, positional indexing) lives in one Rust engine rather
  than in two near-duplicate shell scripts. The shells still do their own path completion,
  keeping their own quoting rules.
- **Task resolution** - Mirrors poe's: `[tool.poe.tasks]`, recursive `include` files
  (env-var expansion, cycle guard), hidden `_` tasks skipped, first definition wins;
  `include_script` is executed against the venv python directly, skipping poe's startup.
- **Caching** - Fingerprint cache at `.cache/devkit-completions.json` (binary version +
  each source's mtime/size); a corrupt cache is a miss, and the data subcommands never
  exit non-zero — a failing completer would break the shell.
- **Install** - Writes the PowerShell shim to `~/.local/share/devkit/poe-completion.ps1`
  and puts one permanent, content-free line in `$PROFILE` that dot-sources it (also
  removing poe's own slow registration, and any previous devkit line); writes the bash
  completion files for Git Bash and Linux; refuses to overwrite files it didn't generate;
  idempotent.
- **Self-repair** - Each request carries a shim version. A shim older than the binary is
  rewritten in place (atomically) for the next shell, while the current request is still
  answered. A shim from before this package existed calls `devkit complete`, which no
  longer exists; `devkit-complete install` (which `setup-project` runs) replaces it.
- **Shells** - PowerShell and bash only.

## Develop

```sh
uv sync                 # builds the crate into .venv through maturin
cargo test
uv run devkit-complete --version
```

The repository is devkit-managed: `poe setup-project` keeps the shared configuration
current. Release with `poe release`; the wheel goes to SFTPyPI and every project takes the
new version on its next `poe setup-project`.
