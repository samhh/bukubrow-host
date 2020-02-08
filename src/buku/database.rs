use super::types::{BookmarkId, SavedBookmark, UnsavedBookmark, BookmarkFilter, BookmarkMatch};
use crate::matching::{match_bookmark};
pub use rusqlite::Error as DbError;
use rusqlite::{types::ToSql, Connection, Row, NO_PARAMS};
use std::path::PathBuf;

pub trait BukuDatabase {
    fn sync(&mut self) -> Result<usize, DbError>;
    fn get_all_bookmarks(&self) -> &Vec<SavedBookmark>;
    fn get_bookmarks_by_id(&self, ids: Vec<BookmarkId>) -> Vec<&SavedBookmark>;
    fn get_filtered_bookmarks(&self, filter: &BookmarkFilter) -> Vec<BookmarkMatch>;
    fn add_bookmarks(&self, bms: &Vec<UnsavedBookmark>) -> Result<Vec<usize>, DbError>;
    fn update_bookmarks(&self, bms: &Vec<SavedBookmark>) -> Result<Vec<usize>, DbError>;
    fn delete_bookmarks(&self, bm_id: &Vec<BookmarkId>) -> Result<Vec<usize>, DbError>;
}

pub struct SqliteDatabase {
    // It would probably be better not to store these in memory and instead
    // simply always query the database, however I'm unsure how to model
    // complex filtering in SQL queries, particularly in a fuzzy manner
    bookmarks: Vec<SavedBookmark>,
    connection: Connection,
}

impl SqliteDatabase {
    // Initiate connection to Sqlite database at specified path
    pub fn new(path: &PathBuf) -> Result<Self, DbError> {
        let connection = Connection::open(&path)?;

        let instance = SqliteDatabase {
            bookmarks: Vec::new(),
            connection,
        };

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
    fn sync(&mut self) -> Result<usize, DbError> {
        let mut stmt = self.connection.prepare("SELECT * FROM bookmarks;")?;

        self.bookmarks = stmt
            .query_map(NO_PARAMS, map_db_bookmark)?
            .filter_map(|bm| bm.ok())
            .collect();

        Ok(self.bookmarks.len())
    }

    fn get_all_bookmarks(&self) -> &Vec<SavedBookmark> {
        &self.bookmarks
    }

    fn get_bookmarks_by_id(&self, ids: Vec<BookmarkId>) -> Vec<&SavedBookmark> {
        self.bookmarks.iter().filter(|bm| ids.contains(&bm.id)).collect()
    }

    fn get_filtered_bookmarks(&self, filter: &BookmarkFilter) -> Vec<BookmarkMatch> {
        self.bookmarks.iter().filter_map(|bm| match_bookmark(&filter, &bm)).collect()

        // TODO this is how we were trying to directly do it in SQL queries before
        // let query = format!(
        //     "SELECT * FROM bookmarks {} AND {} AND {} AND {}",
        //     format!("WHERE metadata = {}", filter.name.as_ref().unwrap_or(&String::from("metadata"))),
        //     format!("WHERE desc = {}", filter.desc.as_ref().unwrap_or(&String::from("desc"))),
        //     // TODO remember, in db tags is comma-separated array
        //     match &filter.tags {
        //         // TODO this is not gonna work
        //         Some(tags) => format!("WHERE tags in ({})", &tags.join(", ")),
        //         None => String::from("WHERE tags = tags"),
        //     },
        //     format!("WHERE url = {}", filter.url.as_ref().unwrap_or(&String::from("url"))),
        //     // TODO wildcard
        //     // with_optional_where("TODOTODOTODO", &filter.wildcard),
        // );

        // let bms = self.connection.prepare(&query)?
        //     .query_map(&[&filter.name], map_db_bookmark)?
        //     .filter_map(|bm| bm.ok())
        //     .collect();

        // Ok(bms)
    }

    fn add_bookmarks(&self, bms: &Vec<UnsavedBookmark>) -> Result<Vec<usize>, DbError> {
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
                        &bm.flags as &ToSql,
                    ],
                )
            })
            .collect()
    }

    fn update_bookmarks(&self, bms: &Vec<SavedBookmark>) -> Result<Vec<usize>, DbError> {
        bms
            .iter()
            .map(|bm| {
                let query = "UPDATE bookmarks SET (metadata, desc, tags, url, flags) = (?2, ?3, ?4, ?5, ?6) WHERE id = ?1;";
                self.connection.execute(
                    query,
                    &[
                        &bm.id,
                        &bm.metadata as &ToSql,
                        &bm.desc,
                        &bm.tags,
                        &bm.url,
                        &bm.flags,
                    ],
                )
            })
            .collect()
    }

    fn delete_bookmarks(&self, bm_ids: &Vec<BookmarkId>) -> Result<Vec<usize>, DbError> {
        bm_ids
            .iter()
            .map(|bm_id| {
                let query = "DELETE FROM bookmarks WHERE id = ?1;";
                self.connection.execute(query, &[bm_id])
            })
            .collect()
    }
}

