pub type BookmarkId = u32;

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Deserialize)]
pub struct BookmarkFilter {
    pub name: Option<String>,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub wildcard: Option<Vec<String>>,
}

// TODO consider combining this inline with bookmark in transmission? and tags could be improved
// eh?
#[derive(Serialize)]
pub struct BookmarkMatch {
    name: Option<Vec<usize>>,
    desc: Option<Vec<usize>>,
    url: Option<Vec<usize>>,
    tags: Option<(i16, Vec<usize>)>,
}

