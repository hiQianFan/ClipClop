# Database compatibility

The application version is currently 0.11.1. Storage versions are separate:
this change migrates schema 4–9 to schema 10 and introduces the compatibility
contract. Previously shipped binaries do not understand this contract and
must be updated once. Do not publish changed database behavior under an
already released application version; increment the application version at release.

## Opening a database

- `PRAGMA user_version` records the migration version. Never change SQLite's
  own `schema_version`, or lower `user_version` to force a downgrade.
- `db_meta.min_compatible_version` (the row keyed `min_compatible_version`)
  is the oldest storage implementation allowed to read, write, and clean up
  the database. Both values start at 10 for this contract.
- A future database with a compatible minimum can be used without migration;
  its version and extra columns remain intact. Required columns are checked.
- Newer incompatible, malformed, unsupported legacy, or unversioned nonempty
  databases are rejected without deletion. The main window shows an error
  page with download, log, and quit actions; capture and cleanup do not start.
- A file lock is held for the lifetime of each file-backed Database. Existing
  single-instance behavior still activates the current app. This does not
  isolate dev data, and does not make older binaries or external SQL tools
  participate in the new lock protocol. Close them before switching builds.

## Adding migrations

Keep `COMPATIBILITY_METADATA_VERSION` fixed at 10. Increment `SCHEMA_VERSION`
and add the corresponding step in `migrations.rs`. `schema.sql` is the schema 9
data-table baseline; new databases run the subsequent migrations in the same
transaction as creation. Do not add a new column there as well as in a later
migration, or it will be created twice.

Keep `MIN_COMPATIBLE_VERSION` unchanged only when older operations remain safe.
Nullable display metadata may qualify; extra required columns, new enum values,
constraints, and changed deletion/retention semantics need review. Favorites
are an example: pre-favorites cleanup can delete entire favorited rows even
though it never references the new column. Raise the minimum when that behavior
is unsafe. SQL preparation alone does not prove business compatibility.

All steps and version updates commit in one IMMEDIATE transaction. Failed steps
roll back the entire chain. Do not open nested transactions inside migration
steps. The version is rechecked after acquiring the transaction. Normal compatible
opens do not run migration integrity scans or create additional backups.

## Backups and recovery

Before migrating an existing file, the SQLite backup API creates and verifies
a snapshot, including committed WAL data, in `clipclop.backups` beside
`clipclop.db`. Filenames contain a timestamp-ordered UUID and the source schema
version. `.partial` files are not completed backups. Backup failure stops migration.
After a successful migration, the latest two completed snapshots are retained;
failed upgrades retain their backups for diagnosis. Unix backup files use mode 0600.

Backups contain clipboard history and can retain records removed from the live
database. They stay local. Incomplete files from failed attempts may be removed
manually after confirming a valid snapshot exists. Backups do not copy external
files referenced by file clipboard entries.

There is no automatic downgrade or in-app restore in this change. Prefer updating
the application when the database is too new. If a backup restore is necessary,
close all instances, preserve the current database and any WAL/SHM files together,
validate the chosen snapshot, and restore it with SQLite-aware tooling. Never
overwrite an open database or combine a restored database with an old WAL file.
Restoring a snapshot rolls back history, favorites, and settings created after
its timestamp. Keep the original data until recovery has been verified.

## Validation

Run `cargo test --manifest-path src-tauri/Cargo.toml` and `pnpm test`.
Fixed historical fixtures cover released schemas 4, 5, 6, 8, and 9, with an
additional schema 7 transition test. Tests cover future-compatible reads/writes,
favorite retention, unknown settings, rollback, WAL snapshots, restore, backup
failure and rotation, file locking, and the frontend startup gate. Run the
existing macOS and Windows Quality jobs before release. Future compatibility
claims should additionally be exercised against the fixed prior implementation;
simulating a future schema version alone does not prove a real future feature safe.
