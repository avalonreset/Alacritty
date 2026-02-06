# Fork Notes (avalonreset/Alacritty)

This repository is an **unofficial fork** of upstream Alacritty:

- Upstream: `alacritty/alacritty`
- This fork: `avalonreset/Alacritty`

The goal is to keep changes small, Windows-focused, and easy to rebase/sync.

## What’s Different Here

This fork carries a small set of quality-of-life patches. At the time of writing, notable changes include:

- Paste image content from the Windows clipboard on `Ctrl+V` (where supported).
- Paste undo on `Ctrl+Z`, plus redo on `Ctrl+Shift+Z` (fork-specific).
- Extra scrolling bindings (`PageUp`/`PageDown`) outside the alt screen (fork-specific).

For exact deltas, check the git history on `main`.

## Keeping In Sync With Upstream

Your git remotes should look like:

- `origin`: your fork (`https://github.com/avalonreset/Alacritty.git`)
- `upstream`: upstream (`https://github.com/alacritty/alacritty.git`)

Typical sync flow:

```bash
git fetch upstream
git checkout main
git merge upstream/master
git push
```

If you prefer rebasing:

```bash
git fetch upstream
git checkout main
git rebase upstream/master
git push --force-with-lease
```

