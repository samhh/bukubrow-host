use super::types::{BookmarkId, SavedBookmark, UnsavedBookmark};
pub use rusqlite::Error as DbError;
use rusqlite::{types::ToSql, Connection, Row, NO_PARAMS};
use std::path::Path;

pub trait BukuDatabase {
    fn get_all_bookmarks(&self) -> Result<Vec<SavedBookmark>, DbError>;
    fn get_bookmarks_by_id(&self, ids: Vec<BookmarkId>) -> Result<Vec<SavedBookmark>, DbError>;
    fn add_bookmarks(&self, bms: &[UnsavedBookmark]) -> Result<Vec<usize>, DbError>;
    fn update_bookmarks(&self, bms: &[SavedBookmark]) -> Result<Vec<usize>, DbError>;
    fn delete_bookmarks(&self, bm_id: &[BookmarkId]) -> Result<Vec<usize>, DbError>;
}

pub struct SqliteDatabase {
    connection: Connection,
}

impl SqliteDatabase {
    // Initiate connection to Sqlite database at specified path
    pub fn new(path: &Path) -> Result<Self, DbError> {
        let connection = Connection::open(&path)?;

        let instance = SqliteDatabase { connection };

        Ok(instance)
    }
}

// Supply defaults for nullable fields (per SQLite schema)
fn map_db_bookmark(row: &Row) -> Result<SavedBookmark, DbError> {
    Ok(SavedBookmark {
        id: row.get(0)?,
        url: row.get(1).unwrap_or_default(),
        metadata: row.get(2).unwrap_or_default(),
        tags: row.get(3).unwrap_or_default(),
        desc: row.get(4).unwrap_or_default(),
        flags: row.get(5).unwrap_or_default(),
    })
}

impl BukuDatabase for SqliteDatabase {
    fn get_all_bookmarks(&self) -> Result<Vec<SavedBookmark>, DbError> {
        let query = "SELECT * FROM bookmarks;";
        let mut stmt = self.connection.prepare(query)?;

        let bookmarks = stmt
            .query_map(NO_PARAMS, map_db_bookmark)?
            .filter_map(|bm| bm.ok())
            .collect();

        Ok(bookmarks)
    }

    fn get_bookmarks_by_id(&self, ids: Vec<BookmarkId>) -> Result<Vec<SavedBookmark>, DbError> {
        let query = format!(
            "SELECT * FROM bookmarks WHERE id IN ({});",
            ids.iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        let mut stmt = self.connection.prepare(&query)?;

        let bookmarks = stmt
            .query_map(NO_PARAMS, map_db_bookmark)?
            .filter_map(|bm| bm.ok())
            .collect();

        Ok(bookmarks)
    }

    fn add_bookmarks(&self, bms: &[UnsavedBookmark]) -> Result<Vec<usize>, DbError> {
        bms
            .iter()
            .map(|bm| {
                let query =
                    "INSERT INTO bookmarks(metadata, desc, tags, url, flags) VALUES (?1, ?2, ?3, ?4, ?5);";
                self.connection.execute(
                    query,
                    &[
                        &bm.metadata,
                        &bm.desc,
                        &bm.tags,
                        &bm.url,
                        &bm.flags as &dyn ToSql,
                    ],
                )
            })
            .collect()
    }

    fn update_bookmarks(&self, bms: &[SavedBookmark]) -> Result<Vec<usize>, DbError> {
        bms
            .iter()
            .map(|bm| {
                let query = "UPDATE bookmarks SET (metadata, desc, tags, url, flags) = (?2, ?3, ?4, ?5, ?6) WHERE id = ?1;";
                self.connection.execute(
                    query,
                    &[
                        &bm.id,
                        &bm.metadata as &dyn ToSql,
                        &bm.desc,
                        &bm.tags,
                        &bm.url,
                        &bm.flags,
                    ],
                )
            })
            .collect()
    }

    fn delete_bookmarks(&self, bm_ids: &[BookmarkId]) -> Result<Vec<usize>, DbError> {
        bm_ids
            .iter()
            .map(|bm_id| {
                let query = "DELETE FROM bookmarks WHERE id = ?1;";
                self.connection.execute(query, &[bm_id])
            })
            .collect()
    }
}
