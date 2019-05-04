pub type BookmarkId = i32;

#[derive(Serialize, Deserialize)]
pub struct Bookmark {
    pub id: Option<BookmarkId>,
    pub url: String,
    pub metadata: String,
    pub tags: String,
    pub desc: String,
    pub flags: i32,
}
