use super::types::{Bookmark, BookmarkId};
pub use rusqlite::Error as DbError;
use rusqlite::{types::ToSql, Connection, Row, NO_PARAMS};
use std::path::PathBuf;

pub trait BukuDatabase {
    fn get_all_bookmarks(&self) -> Result<Vec<Bookmark>, DbError>;
    fn get_bookmarks_by_id(&self, ids: Vec<BookmarkId>) -> Result<Vec<Bookmark>, DbError>;
    fn add_bookmark(&self, bm: &Bookmark) -> Result<usize, DbError>;
    fn update_bookmark(&self, bm: &Bookmark) -> Result<usize, DbError>;
    fn delete_bookmark(&self, bm_id: &BookmarkId) -> Result<usize, DbError>;
}

pub struct SqliteDatabase {
    connection: Connection,
}

impl SqliteDatabase {
    // Initiate connection to Sqlite database at specified path
    pub fn new(path: &PathBuf) -> Result<Self, DbError> {
        let connection = Connection::open(&path)?;

        let instance = SqliteDatabase { connection };

        Ok(instance)
    }
}

// Supply defaults for nullable fields (per SQLite schema)
fn map_db_bookmark(row: &Row) -> Result<Bookmark, DbError> {
    Ok(Bookmark {
        id: row.get(0)?,
        url: row.get(1).unwrap_or_default(),
        metadata: row.get(2).unwrap_or_default(),
        tags: row.get(3).unwrap_or_default(),
        desc: row.get(4).unwrap_or_default(),
        flags: row.get(5).unwrap_or_default(),
    })
}

impl BukuDatabase for SqliteDatabase {
    // Get bookmarks from database
    fn get_all_bookmarks(&self) -> Result<Vec<Bookmark>, DbError> {
        let query = "SELECT * FROM bookmarks;";
        let mut stmt = self.connection.prepare(query)?;

        let rows = stmt.query_map(NO_PARAMS, map_db_bookmark)?;

        let bookmarks: Vec<Bookmark> = rows.filter_map(|x| x.ok()).collect();

        Ok(bookmarks)
    }

    fn get_bookmarks_by_id(&self, ids: Vec<BookmarkId>) -> Result<Vec<Bookmark>, DbError> {
        let query = format!(
            "SELECT * FROM bookmarks WHERE id IN ({});",
            ids.iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt.query_map(NO_PARAMS, map_db_bookmark)?;

        let bookmarks: Vec<Bookmark> = rows.filter_map(|x| x.ok()).collect();

        Ok(bookmarks)
    }

    // Save bookmark to database
    fn add_bookmark(&self, bm: &Bookmark) -> Result<usize, DbError> {
        let query =
            "INSERT INTO bookmarks(metadata, desc, tags, url, flags) VALUES (?1, ?2, ?3, ?4, ?5);";
        let exec = self.connection.execute(
            query,
            &[
                &bm.metadata,
                &bm.desc,
                &bm.tags,
                &bm.url,
                &bm.flags as &ToSql,
            ],
        );

        exec
    }

    // Update bookmark in database by ID
    fn update_bookmark(&self, bm: &Bookmark) -> Result<usize, DbError> {
        let query = "UPDATE bookmarks SET (metadata, desc, tags, url, flags) = (?2, ?3, ?4, ?5, ?6) WHERE id = ?1;";
        let exec = self.connection.execute(
            query,
            &[
                &bm.id.unwrap(),
                &bm.metadata as &ToSql,
                &bm.desc,
                &bm.tags,
                &bm.url,
                &bm.flags,
            ],
        );

        exec
    }

    // Delete bookmark from database by ID
    fn delete_bookmark(&self, bm_id: &BookmarkId) -> Result<usize, DbError> {
        let query = "DELETE FROM bookmarks WHERE id = ?1;";
        let exec = self.connection.execute(query, &[bm_id]);

        exec
    }
}
