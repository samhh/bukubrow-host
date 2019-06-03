use crate::buku::database::BukuDatabase;
use crate::buku::types::{BookmarkId, SavedBookmark, UnsavedBookmark};
use crate::native_messaging::{read_input, write_output, NativeMessagingError};
use clap::crate_version;
use std::io;

/// If the server is not provided with a valid database, it needs to know why
/// so that it can communicate that.
pub enum InitError {
    FailedToLocateBukuDatabase,
    FailedToAccessBukuDatabase,
}

pub fn map_init_err_friendly_msg(err: &InitError) -> &'static str {
    match err {
        InitError::FailedToLocateBukuDatabase => "Failed to locate Buku database.",
        InitError::FailedToAccessBukuDatabase => "Failed to access Buku database.",
    }
}

type JSON = serde_json::Value;

#[derive(Debug, PartialEq)]
enum Method {
    Get,
    Options,
    Post,
    Put,
    Delete,
    UnknownMethod,
    NoMethod,
}

#[derive(Deserialize)]
struct RequestMethod {
    method: String,
}

#[derive(Deserialize)]
struct RequestData<T> {
    data: T,
}

#[derive(Deserialize)]
struct RequestDataPost {
    bookmarks: Vec<UnsavedBookmark>,
}

type PostRequest = RequestData<RequestDataPost>;

#[derive(Deserialize)]
struct RequestDataPut {
    bookmarks: Vec<SavedBookmark>,
}

type PutRequest = RequestData<RequestDataPut>;

#[derive(Deserialize)]
struct RequestDataDelete {
    bookmark_ids: Vec<BookmarkId>,
}

type DeleteRequest = RequestData<RequestDataDelete>;

pub struct Server<T> {
    db: Result<T, InitError>,
}

impl<T: BukuDatabase> Server<T> {
    pub fn new(db: Result<T, InitError>) -> Self {
        Self { db }
    }

    // Listen for native messages from WebExtension in a loop
    pub fn listen(&self) -> Result<(), NativeMessagingError> {
        loop {
            match read_input(io::stdin()) {
                Ok(payload) => {
                    let res = self.router(payload);
                    write_output(io::stdout(), &res)?;
                }
                Err(err) => match err {
                    NativeMessagingError::NoMoreInput => break Err(err),
                    _ => {}
                },
            }
        }
    }

    fn method_deserializer(&self, payload: JSON) -> Method {
        if let Ok(RequestMethod { method }) = serde_json::from_value(payload) {
            match method.as_ref() {
                "GET" => Method::Get,
                "OPTIONS" => Method::Options,
                "POST" => Method::Post,
                "PUT" => Method::Put,
                "DELETE" => Method::Delete,
                _ => Method::UnknownMethod,
            }
        } else {
            Method::NoMethod
        }
    }

    // Route requests per the method
    pub fn router(&self, payload: JSON) -> JSON {
        match &self.db {
            Ok(db) => match self.method_deserializer(payload.clone()) {
                Method::Get => self.get(&db),
                Method::Options => self.options(),
                Method::Post => serde_json::from_value::<PostRequest>(payload)
                    .map(|req| self.post(&db, &req.data.bookmarks))
                    .unwrap_or_else(|_| self.fail_bad_payload()),
                Method::Put => serde_json::from_value::<PutRequest>(payload)
                    .map(|req| self.put(&db, &req.data.bookmarks))
                    .unwrap_or_else(|_| self.fail_bad_payload()),
                Method::Delete => serde_json::from_value::<DeleteRequest>(payload)
                    .map(|req| self.delete(&db, &req.data.bookmark_ids))
                    .unwrap_or_else(|_| self.fail_bad_payload()),
                Method::UnknownMethod => self.fail_unknown_method(),
                Method::NoMethod => self.fail_no_method(),
            },
            Err(err) => self.fail_init_error(err),
        }
    }

    fn get(&self, db: &T) -> JSON {
        let bookmarks = db.get_all_bookmarks();

        match bookmarks {
            Ok(bm) => json!({
                "success": true,
                "bookmarks": bm,
            }),
            Err(_) => self.fail_generic(),
        }
    }

    fn options(&self) -> JSON {
        json!({
            "success": true,
            "binaryVersion": crate_version!(),
        })
    }

    fn post(&self, db: &T, bms: &Vec<UnsavedBookmark>) -> JSON {
        let added = db.add_bookmarks(&bms);

        if let Ok(ids) = added {
            json!({
                "success": true,
                "ids": ids,
            })
        } else {
            self.fail_generic()
        }
    }

    fn put(&self, db: &T, bms: &Vec<SavedBookmark>) -> JSON {
        let update = db.update_bookmarks(&bms);

        json!({ "success": update.is_ok() })
    }

    fn delete(&self, db: &T, bm_ids: &Vec<BookmarkId>) -> JSON {
        let deletion = db.delete_bookmarks(&bm_ids);

        json!({ "success": deletion.is_ok() })
    }

    fn fail_generic(&self) -> JSON {
        json!({ "success": false })
    }

    fn fail_no_method(&self) -> JSON {
        json!({
            "success": false,
            "message": "Missing method type.",
        })
    }

    fn fail_unknown_method(&self) -> JSON {
        json!({
            "success": false,
            "message": "Unrecognised method type.",
        })
    }

    fn fail_bad_payload(&self) -> JSON {
        json!({
            "success": false,
            "message": "Bad request payload."
        })
    }

    fn fail_init_error(&self, err: &InitError) -> JSON {
        json!({
            "success": false,
            "message": map_init_err_friendly_msg(err),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buku::database::{BukuDatabase, DbError, SqliteDatabase};

    fn shared_mock_update_ids() -> Vec<usize> {
        vec![1, 2, 3, 4]
    }

    fn create_mocked_server() -> Server<impl BukuDatabase> {
        struct BukuMock {}

        impl BukuDatabase for BukuMock {
            fn get_all_bookmarks(&self) -> Result<Vec<SavedBookmark>, DbError> {
                Ok(Vec::new())
            }

            fn get_bookmarks_by_id(
                &self,
                _ids: Vec<BookmarkId>,
            ) -> Result<Vec<SavedBookmark>, DbError> {
                Ok(Vec::new())
            }

            fn add_bookmarks(&self, _bm: &Vec<UnsavedBookmark>) -> Result<Vec<usize>, DbError> {
                Ok(shared_mock_update_ids())
            }

            fn update_bookmarks(&self, _bm: &Vec<SavedBookmark>) -> Result<Vec<usize>, DbError> {
                Ok(shared_mock_update_ids())
            }

            fn delete_bookmarks(&self, _bm_ids: &Vec<BookmarkId>) -> Result<Vec<usize>, DbError> {
                Ok(shared_mock_update_ids())
            }
        }

        Server {
            db: Ok(BukuMock {}),
        }
    }

    fn create_mocked_server_with_init_err(err: InitError) -> Server<SqliteDatabase> {
        Server { db: Err(err) }
    }

    fn create_example_saved_bookmarks() -> Vec<SavedBookmark> {
        vec![
            SavedBookmark {
                id: 0,
                url: String::from("https://samhh.com"),
                metadata: String::from("title"),
                tags: String::from(""),
                desc: String::from("description"),
                flags: 0,
            }
        ]
    }

    fn create_example_unsaved_bookmarks() -> Vec<UnsavedBookmark> {
        vec![
            UnsavedBookmark {
                url: String::from("https://samhh.com"),
                metadata: String::from("title"),
                tags: String::from(""),
                desc: String::from("description"),
                flags: 0,
            }
        ]
    }

    #[test]
    fn test_method_deserializer() {
        let server = create_mocked_server();

        assert_eq!(
            server.method_deserializer(json!({ "method": "GET" })),
            Method::Get
        );

        assert_eq!(
            server.method_deserializer(json!({ "method": "get" })),
            Method::UnknownMethod
        );

        assert_eq!(server.method_deserializer(json!({})), Method::NoMethod);

        assert_eq!(
            server.method_deserializer(json!({ "other": "property" })),
            Method::NoMethod
        );
    }

    #[test]
    fn test_router_with_locate_init_error() {
        let server_failed_locating =
            create_mocked_server_with_init_err(InitError::FailedToLocateBukuDatabase);

        assert_eq!(
            server_failed_locating.router(json!({ "method": "GET" })),
            server_failed_locating.fail_init_error(&InitError::FailedToLocateBukuDatabase),
        );
    }

    #[test]
    fn test_router_with_access_init_error() {
        let server_failed_locating =
            create_mocked_server_with_init_err(InitError::FailedToAccessBukuDatabase);

        assert_eq!(
            server_failed_locating.router(json!({ "method": "GET" })),
            server_failed_locating.fail_init_error(&InitError::FailedToAccessBukuDatabase),
        );
    }

    #[test]
    fn test_router_get() {
        let server = create_mocked_server();

        assert_eq!(
            server.router(json!({ "method": "GET" })),
            json!({ "success": true, "bookmarks": Vec::<SavedBookmark>::new() }),
        );
    }

    #[test]
    fn test_router_options() {
        let server = create_mocked_server();

        assert_eq!(
            server.router(json!({ "method": "OPTIONS" })),
            json!({ "success": true, "binaryVersion": crate_version!() })
        );
    }

    #[test]
    fn test_router_post() {
        let server = create_mocked_server();

        assert_eq!(
            server.router(json!({ "method": "POST" })),
            server.fail_bad_payload(),
        );

        assert_eq!(
            server.router(json!({
                "method": "POST",
                "data": {
                    "bookmarks": create_example_unsaved_bookmarks(),
                },
            })),
            json!({
                "success": true,
                "ids": shared_mock_update_ids(),
            }),
        );
    }

    #[test]
    fn test_router_put() {
        let server = create_mocked_server();

        assert_eq!(
            server.router(json!({ "method": "PUT" })),
            server.fail_bad_payload(),
        );

        assert_eq!(
            server.router(json!({
                "method": "PUT",
                "data": {
                    "bookmarks": create_example_saved_bookmarks(),
                },
            })),
            json!({ "success": true }),
        );
    }

    #[test]
    fn test_router_delete() {
        let server = create_mocked_server();

        assert_eq!(
            server.router(json!({ "method": "DELETE" })),
            server.fail_bad_payload(),
        );

        assert_eq!(
            server.router(json!({
                "method": "DELETE",
                "bookmarkId": 99,
            })),
            server.fail_bad_payload(),
        );

        assert_eq!(
            server.router(json!({
                "method": "DELETE",
                "bookmark_id": 99,
            })),
            server.fail_bad_payload(),
        );

        assert_eq!(
            server.router(json!({
                "method": "DELETE",
                "data": {
                    "bookmark_id": 99,
                },
            })),
            server.fail_bad_payload(),
        );

        assert_eq!(
            server.router(json!({
                "method": "DELETE",
                "data": {
                    "bookmark_ids": vec![99],
                },
            })),
            json!({ "success": true }),
        );
    }
}
