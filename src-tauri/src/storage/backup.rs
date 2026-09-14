use std::{fs, path::Path};

use rusqlite::{Connection, OpenFlags};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

pub(super) fn create(connection: &Connection, path: &Path, version: u32) -> AppResult<()> {
    let directory = path.with_extension("backups");
    fs::create_dir_all(&directory).map_err(storage_error)?;
    let name = format!("{}-v{version}", Uuid::now_v7());
    let partial = directory.join(format!("{name}.partial"));
    let completed = directory.join(format!("{name}.sqlite"));
    let mut options = fs::OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(&partial).map_err(storage_error)?;
    // SQLite's backup API includes committed WAL pages; copying just the .db does not.
    connection.backup(rusqlite::MAIN_DB, &partial, None)?;
    let snapshot = Connection::open(&partial)?;
    snapshot.pragma_update(None, "journal_mode", "DELETE")?;
    verify(&snapshot)?;
    snapshot
        .close()
        .map_err(|(_, error)| AppError::from(error))?;
    file.sync_all().map_err(storage_error)?;
    drop(file);
    fs::rename(&partial, &completed).map_err(storage_error)?;
    #[cfg(unix)]
    fs::File::open(&directory)
        .and_then(|directory| directory.sync_all())
        .map_err(storage_error)?;
    log::info!(
        "database migration backup completed: {}",
        completed.display()
    );
    Ok(())
}

pub(super) fn verify(connection: &Connection) -> AppResult<()> {
    let result: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if result != "ok" || connection.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(AppError::DatabaseInvalid("integrity check failed".into()));
    }
    Ok(())
}

// Only rotate after a successful migration. Failed attempts cannot evict a good backup.
pub(super) fn prune(path: &Path) -> AppResult<()> {
    let directory = path.with_extension("backups");
    let mut backups = fs::read_dir(directory)
        .map_err(storage_error)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(storage_error)?
        .into_iter()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sqlite"))
        .filter(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.rsplit_once("-v"))
                .is_some_and(|(id, version)| {
                    Uuid::parse_str(id).is_ok() && version.parse::<u32>().is_ok()
                })
        })
        .collect::<Vec<_>>();
    backups.sort();
    let remove_count = backups.len().saturating_sub(2);
    for path in backups.into_iter().take(remove_count) {
        // Never remove unverified files automatically.
        let snapshot = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        verify(&snapshot)?;
        drop(snapshot);
        fs::remove_file(path).map_err(storage_error)?;
    }
    Ok(())
}

pub(super) fn storage_error(error: std::io::Error) -> AppError {
    AppError::Storage(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backups_restore_and_rotation_preserves_unrelated_and_incomplete_files() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("clips.db");
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(include_str!("fixtures/v9.sql"))
            .unwrap();
        connection
            .execute("INSERT INTO settings VALUES('sentinel','123')", [])
            .unwrap();
        for _ in 0..3 {
            create(&connection, &path, 9).unwrap();
        }
        let directory = path.with_extension("backups");
        let unrelated = directory.join("personal.sqlite");
        let partial = directory.join("unfinished.partial");
        fs::write(&unrelated, "leave this alone").unwrap();
        fs::write(&partial, "incomplete").unwrap();
        prune(&path).unwrap();
        assert!(unrelated.exists());
        assert!(partial.exists());
        let snapshots: Vec<_> = fs::read_dir(&directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|file| {
                file != &unrelated && file.extension().is_some_and(|ext| ext == "sqlite")
            })
            .collect();
        assert_eq!(snapshots.len(), 2);
        let mut restored = Connection::open(temp.path().join("restored.db")).unwrap();
        restored
            .restore(
                rusqlite::MAIN_DB,
                &snapshots[0],
                None::<fn(rusqlite::backup::Progress)>,
            )
            .unwrap();
        verify(&restored).unwrap();
        assert_eq!(
            restored
                .pragma_query_value(None, "user_version", |row| row.get::<_, u32>(0))
                .unwrap(),
            9
        );
        assert_eq!(
            restored
                .query_row(
                    "SELECT value_json FROM settings WHERE key='sentinel'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "123"
        );
    }
}
