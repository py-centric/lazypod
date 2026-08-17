---
description: "Git branching and synchronization rules for Lazypod"
globs: ["**/*"]
always_on: true
---

# Git Workflow & Synchronization Rules

1. **Rebase Only**:
   - When synchronizing feature branches or worktrees to `main`, ALWAYS use `git rebase <branch>` instead of `git merge` or `git merge --ff-only`.
   - Never create merge commits in `main`.
2. **Conventional Commits**:
   - Use atomic conventional commits with standard types (`feat`, `fix`, `perf`, `refactor`, `test`, `docs`, `chore`).
   - Group changes logically by component (`podman`, `app`, `ui`).
