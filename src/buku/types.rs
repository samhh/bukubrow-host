pub type BookmarkId = u32;

#[derive(Serialize, Deserialize)]
pub struct SavedBookmark {
    pub id: BookmarkId,
    pub url: String,
    pub metadata: String,
    pub tags: String,
    pub desc: String,
    pub flags: i32,
}

#[derive(Serialize, Deserialize)]
pub struct UnsavedBookmark {
    pub url: String,
    pub metadata: String,
    pub tags: String,
    pub desc: String,
    pub flags: i32,
}
