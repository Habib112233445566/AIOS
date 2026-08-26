# 🤖 AIOS Workspace Instructions for AI Agents

You are operating inside a persistent, R2-backed AIOS workspace. 
Your actions affect the local `/content/AIOS_MERGED` directory, which is synchronized with Cloudflare R2.

## 🛑 CRITICAL RULES
1. **ALWAYS snapshot before massive changes:** Before deleting, moving, or refactoring hundreds of files, run:
   `python workspace.py snapshot`
2. **NEVER try to manually manipulate R2 objects:** Do not use `aws s3 rm`, `bun`, or direct API calls to delete files from R2. The workspace tool handles this safely and preserves history.
3. **Deletions are local-first:** To delete files, delete them locally, then run `sync`. The tool will update the remote manifest to reflect deletions while keeping immutable blobs safe for historical snapshots.

## 🛠️ AVAILABLE WORKSPACE COMMANDS
Run these from the `/content` directory:

- **Check Status:** `python workspace.py status`
  *(Use this to verify how many files are tracked and how many snapshots exist).*

- **Save Changes (Sync):** `python workspace.py sync /content/AIOS_MERGED`
  *(Run this AFTER you create, edit, or delete files. It will only upload changed files and update the remote manifest to reflect local deletions).*

- **Create Checkpoint:** `python workspace.py snapshot`
  *(Creates an instant, metadata-only backup of the current state. Takes < 1 second).*

- **Restore Checkpoint:** `python workspace.py restore <snapshot_id> /content/RESTORE_DIR`
  *(Use this to recover files if a deletion or refactor goes wrong).*

## 📝 EXAMPLE WORKFLOW: Deleting 500+ Files
If asked to delete a large number of files, follow this exact sequence:
1. `python workspace.py snapshot` (Create a safe restore point)
2. `rm -rf /content/AIOS_MERGED/path/to/delete` (Or use a python/bash script to delete the files locally)
3. `python workspace.py sync /content/AIOS_MERGED` (This will report "X deleted locally" and update R2)
4. `python workspace.py status` (Verify the new file count)
