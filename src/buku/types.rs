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

// The first item represents the quality of the match, and the second item
// represents the indices that matched
pub type MatchDetails = (i64, Vec<usize>);
pub type MatchDetailsMulti = (i64, Vec<(i32, Vec<usize>)>);

// TODO consider combining this inline with bookmark in transmission? and tags could be improved
// eh?
#[derive(Serialize)]
pub struct BookmarkMatch {
    pub name: Option<MatchDetails>,
    pub desc: Option<MatchDetails>,
    pub url: Option<MatchDetails>,
    pub tags: Option<MatchDetailsMulti>,
}

