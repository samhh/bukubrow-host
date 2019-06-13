use crate::buku::database::BukuDatabase;
use crate::buku::types::{BookmarkId, SavedBookmark, UnsavedBookmark};
use crate::native_messaging::{read_input, write_output, NativeMessagingError, ONE_MEGABYTE_BYTES};
use clap::crate_version;
use std::io;

/// If the server is not provided with a valid database, it needs to know why
/// so that it can communicate that.
pub enum InitError {
    FailedToLocateBukuDatabase,
    FailedToAccessBukuDatabase,
}

#[derive(Debug, PartialEq)]
enum BookmarksSplitError {
    BookmarkLargerThanMaxPayloadSize,
    Unknown,
}

enum BookmarksSplitOffset {
    Offset(usize),
    None,
}

impl BookmarksSplitOffset {
    fn unwrap_or(&self, v: usize) -> usize {
        match self {
            BookmarksSplitOffset::Offset(offset) => *offset,
            BookmarksSplitOffset::None => v,
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq)]
enum BookmarksSplitPayloadSize {
    Limited(usize),
    Unlimited,
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
struct RequestDataGet {
    offset: Option<usize>,
}

type GetRequest = RequestData<Option<RequestDataGet>>;

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
                    write_output(io::stdout(), &self.router(payload))?;
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

    fn split_bookmarks_subset(
        &self,
        all_bms: &Vec<SavedBookmark>,
        bms_offset: BookmarksSplitOffset,
        max_page_size_bytes: BookmarksSplitPayloadSize,
    ) -> Result<JSON, BookmarksSplitError> {
        let gen_res = |bms: &Vec<SavedBookmark>, are_more: bool| {
            json!({
                "success": true,
                "bookmarks": &bms,
                "moreAvailable": are_more,
            })
        };

        if all_bms.is_empty() {
            return Ok(gen_res(&all_bms, false));
        }

        let offset = bms_offset.unwrap_or(0);
        let bms = &all_bms[offset..];

        match max_page_size_bytes {
            BookmarksSplitPayloadSize::Unlimited => Ok(gen_res(&bms.to_vec(), false)),
            BookmarksSplitPayloadSize::Limited(max_size) => {
                let overhead = serde_json::to_vec(&gen_res(&vec![], false))
                    .map_err(|_| BookmarksSplitError::Unknown)?
                    .len();
                let mut size_so_far = overhead;

                for (i, bm) in bms.iter().enumerate() {
                    let bm_size = serde_json::to_vec(&bm)
                        .map_err(|_| BookmarksSplitError::Unknown)?
                        .len();

                    let new_size_so_far = size_so_far + bm_size + std::cmp::min(i, 1); // Comma is 1 byte
                    if new_size_so_far >= max_size {
                        if i == 0 {
                            return Err(BookmarksSplitError::BookmarkLargerThanMaxPayloadSize);
                        }

                        return Ok(gen_res(&all_bms[offset..offset + i].to_vec(), true));
                    }

                    let is_last_loop = i == bms.len() - 1;
                    if is_last_loop {
                        return Ok(gen_res(&bms.to_vec(), false));
                    }

                    size_so_far = new_size_so_far;
                }

                Err(BookmarksSplitError::Unknown)
            }
        }
    }

    // Route requests per the method
    pub fn router(&self, payload: JSON) -> JSON {
        match &self.db {
            Ok(db) => match self.method_deserializer(payload.clone()) {
                Method::Get => serde_json::from_value::<GetRequest>(payload)
                    .map(|req| self.get(&db, &req.data.and_then(|d| d.offset)))
                    .unwrap_or_else(|_| self.fail_bad_payload()),
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

    fn get(&self, db: &T, offset_opt: &Option<usize>) -> JSON {
        let bookmarks = db.get_all_bookmarks();
        let offset = offset_opt.map_or_else(
            || BookmarksSplitOffset::None,
            |o| BookmarksSplitOffset::Offset(o),
        );

        match bookmarks {
            Ok(bms) => self
                .split_bookmarks_subset(
                    &bms,
                    offset,
                    BookmarksSplitPayloadSize::Limited(*ONE_MEGABYTE_BYTES),
                )
                .unwrap_or_else(|_| self.fail_generic()),
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

    fn create_bms(range: impl ExactSizeIterator<Item = u16>) -> Vec<SavedBookmark> {
        let mut bms = Vec::with_capacity(range.len());

        for i in range {
            bms.push(SavedBookmark {
                id: i as u32,
                metadata: String::from(""),
                desc: String::from(""),
                url: String::from(""),
                tags: String::from(""),
                flags: 0,
            });
        }

        bms
    }

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
        vec![SavedBookmark {
            id: 0,
            url: String::from("https://samhh.com"),
            metadata: String::from("title"),
            tags: String::from(""),
            desc: String::from("description"),
            flags: 0,
        }]
    }

    fn create_example_unsaved_bookmarks() -> Vec<UnsavedBookmark> {
        vec![UnsavedBookmark {
            url: String::from("https://samhh.com"),
            metadata: String::from("title"),
            tags: String::from(""),
            desc: String::from("description"),
            flags: 0,
        }]
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
    fn test_split_bookmarks_subset() {
        let server = create_mocked_server();
        let bm_bytes_length = serde_json::to_vec(&create_bms(0..1).pop().unwrap())
            .unwrap()
            .to_vec()
            .len();
        let overhead_bytes_length = serde_json::to_vec(
            &server
                .split_bookmarks_subset(
                    &create_bms(0..1),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(999999),
                )
                .unwrap(),
        )
        .unwrap()
        .len();

        // No limit or offset
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..2),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Unlimited
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": false,
                "bookmarks": create_bms(0..2),
            }),
        );

        // No limit with offset
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..2),
                    BookmarksSplitOffset::Offset(1),
                    BookmarksSplitPayloadSize::Unlimited
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": false,
                "bookmarks": create_bms(1..2),
            }),
        );

        // No bookmarks available
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &Vec::new(),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(overhead_bytes_length)
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": false,
                "bookmarks": create_bms(0..0),
            }),
        );

        // Insufficient space for both bookmark and overhead
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..1),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(overhead_bytes_length)
                )
                .unwrap_err(),
            BookmarksSplitError::BookmarkLargerThanMaxPayloadSize,
        );

        // One bookmark, fitting
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..1),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(bm_bytes_length + overhead_bytes_length)
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": false,
                "bookmarks": create_bms(0..1),
            }),
        );

        // Two bookmarks, both fitting
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..2),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(
                        (bm_bytes_length * 2) + overhead_bytes_length
                    )
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": false,
                "bookmarks": create_bms(0..2),
            }),
        );

        // Three bookmarks, two fitting with one more available
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..3),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(
                        (bm_bytes_length * 2) + overhead_bytes_length
                    )
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": true,
                "bookmarks": create_bms(0..2),
            }),
        );

        // Three bookmarks, two fitting, one offset so two remaining both fit
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..3),
                    BookmarksSplitOffset::Offset(1),
                    BookmarksSplitPayloadSize::Limited(
                        (bm_bytes_length * 2) + overhead_bytes_length
                    )
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": false,
                "bookmarks": create_bms(1..3),
            }),
        );

        // Four bookmarks, two fitting, one offset so two fit with one more available
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..4),
                    BookmarksSplitOffset::Offset(1),
                    BookmarksSplitPayloadSize::Limited(
                        (bm_bytes_length * 2) + overhead_bytes_length
                    )
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": true,
                "bookmarks": create_bms(1..3),
            }),
        );

        // Four bookmarks, two fitting, two more available
        assert_eq!(
            server
                .split_bookmarks_subset(
                    &create_bms(0..4),
                    BookmarksSplitOffset::None,
                    BookmarksSplitPayloadSize::Limited(
                        (bm_bytes_length * 2) + overhead_bytes_length
                    )
                )
                .unwrap(),
            json!({
                "success": true,
                "moreAvailable": true,
                "bookmarks": create_bms(0..2),
            }),
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
            json!({ "success": true, "bookmarks": Vec::<SavedBookmark>::new(), "moreAvailable": false }),
        );

        assert_eq!(
            server.router(json!({ "method": "GET", "data": { "offset": 1 } })),
            json!({ "success": true, "bookmarks": Vec::<SavedBookmark>::new(), "moreAvailable": false }),
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
