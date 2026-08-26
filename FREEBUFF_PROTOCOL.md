# 🤖 AIOS WORKSPACE PROTOCOL FOR AGENTS

You are operating inside a persistent, cloud-backed workspace at `/content/AIOS_MERGED`.

## 🛠️ YOUR PERSISTENCE TOOL: `workspace.py` (run from /content, ALWAYS use --json)
- `python workspace.py status --json` -> Check state.
- `python workspace.py sync /content/AIOS_MERGED --json` -> SAVE CHANGES (uploads edits/creations, removes deletions from manifest).
- `python workspace.py snapshot --json` -> Instant checkpoint before big changes.
- `python workspace.py restore <snapshot_id> /content/RECOVERY --json` -> Recover files.
- `python workspace.py gc --json` -> PREVIEW permanent deletes (safe, deletes nothing).
- `python workspace.py gc --execute --json` -> HARD DELETE (permanently destroys orphaned files from R2).

## 🧠 RULES
1. Do all file work locally in /content/AIOS_MERGED (create/edit/move/delete with normal shell tools).
2. After each task, run sync --json.
3. Before massive refactors or mass deletions, run snapshot --json first.
4. Normal deletions are soft (recoverable). Only use gc --execute when the user explicitly asks to permanently delete or free up space.
5. Never use aws/s3/bun commands against R2 directly. workspace.py is the only bridge.
