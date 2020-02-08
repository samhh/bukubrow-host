use crate::buku::types::{SavedBookmark, BookmarkFilter, BookmarkMatch};
use fuzzy_matcher::skim::SkimMatcherV2;

fn fuzzy_match_string(matcher: &str, search: &str) -> Option<(i64, Vec<usize>)> {
    // fuzzy_indices(matcher, search)
    None
}

/*
/**
 * Filter out bookmarks that do not perfectly match the provided test.
 */
export const filterBookmarks = (bookmarks: Array<LocalBookmark>, test: ParsedInputResult): Array<LocalBookmark> =>
	bookmarks.filter((bookmark) => {
		if (!includesCI(test.name)(bookmark.title)) return false;
		if (test.desc.some(d => !includesCI(d)(bookmark.desc))) return false;
		if (test.url.some(u => !includesCI(u)(bookmark.url))) return false;
		if (test.tags.some(t => !bookmark.tags.some(tag => includesCI(t)(tag)))) return false;

		// Ensure all wildcards match something
		const allWildcardsMatch = test.wildcard.every((wc) => {
			return (
				includesCI(wc)(bookmark.title) ||
				includesCI(wc)(bookmark.desc) ||
				includesCI(wc)(bookmark.url) ||
				bookmark.tags.some(tag => includesCI(wc)(tag))
			);
		});

		return allWildcardsMatch;
	});
*/

// Filter out bookmarks that do not perfectly match the provided test
pub fn match_bookmark(filter: &BookmarkFilter, bm: &SavedBookmark) -> Option<BookmarkMatch> {
    // let a = fuzzy_indices(&filter.name);
    None
}

