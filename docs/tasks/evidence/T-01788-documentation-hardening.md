# Task Evidence: T-01788 - Hardware Detection / Documentation: Hardening

## Metadata
- **Task ID:** `T-01788`
- **Sub-Epic:** Sub-Epic 9: Hardware Detection / Documentation
- **Component:** `aiosh-core::hardware_doc`, `aiosh-core::tests::test_hardware_doc`
- **Date:** 2026-09-20
- **Status:** COMPLETED

## Hardening Implemented
1. **UTF-8 Character Boundary Safe Slicing:**
   - Remediation for `THREAT-HDOC-03`: In `HardwareDocIndex::search`, snippet generation now steps backward to `is_char_boundary(start)` and forward to `is_char_boundary(end)` rather than blindly slicing byte offsets.
   - Prevents panic attacks or server crashes when documentation contains UTF-8 multi-byte glyphs (emojis, accented characters, CJK characters).
2. **Topic ID Sanitization:**
   - In `HardwareDocIndex::get_topic`, restricted topic IDs strictly to ASCII alphanumeric characters, hyphens (`-`), underscores (`_`), and dots (`.`).
   - Rejects any control characters, path separators (`/`, `\`), null bytes, script tags, whitespace, or command injection characters (`;&|`).
3. **Category String Bounds:**
   - In `HardwareDocCategory::from_str_loose`, added an upper limit of 32 characters on inputs to avoid wasting CPU cycles on excessively large strings before matching.
4. **Hardening Verification:**
   - Added unit test `test_hdoc_hardening` covering UTF-8 multi-byte emoji text snippet extraction, invalid character injection rejection, and oversized category parsing.
