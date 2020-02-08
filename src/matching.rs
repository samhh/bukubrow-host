use crate::buku::types::{SavedBookmark, BookmarkFilter, BookmarkMatch, MatchDetails};
use fuzzy_matcher::skim::fuzzy_indices;

enum MatchSuccess {
    Unneeded,
    Required(MatchDetails),
}

fn fuzzy_match_string(testee: &str, search: &String) -> Option<MatchDetails> {
    fuzzy_indices(&testee, &search)
}

// Test against multiple strings and return the best match, if any
fn fuzzy_match_string_multi(testee: &str, searches: &Vec<String>) -> Option<MatchDetails> {
    searches.iter()
        .map(|search| fuzzy_match_string(testee, search))
        .fold(None, |acc, val| match acc {
            None => val,
            Some(accx) => match val {
                None => Some(accx),
                Some(valx) => if valx.0 > accx.0 { Some(valx) } else { Some(accx) }
            }
        })
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

type IndexedMatch = (i32, MatchDetails);

fn reduce_results(results: &Vec<MatchSuccess>) -> Option<Vec<IndexedMatch>> {
    None // TODO
}

fn deconstruct_success(success: MatchSuccess) -> Option<MatchDetails> {
    match success {
        MatchSuccess::Unneeded => None,
        MatchSuccess::Required(dets) => Some(dets),
    }
}

// TODO will need to average these quality nums together, and filter out if any are zero
// TODO there's a change to the parsing here versus frontend where we assume only one desc and one
// url (like name/title)
// Filter out bookmarks that do not perfectly match the provided test
pub fn match_bookmark(filter: &BookmarkFilter, bm: &SavedBookmark) -> Option<BookmarkMatch> {
    let name_match = match &filter.name {
        None => Some(MatchSuccess::Unneeded),
        Some(name) => fuzzy_match_string(&bm.metadata, &name).map(MatchSuccess::Required),
    };
    let desc_match = match &filter.desc {
        None => Some(MatchSuccess::Unneeded),
        Some(desc) => fuzzy_match_string(&bm.desc, &desc).map(MatchSuccess::Required),
    };
    let url_match = match &filter.url {
        None => Some(MatchSuccess::Unneeded),
        Some(url) => fuzzy_match_string(&bm.url, &url).map(MatchSuccess::Required),
    };

    // TODO tags
    // let tags_match = filter.tags.as_ref().and_then(|tags| tags.iter().map(|TODO| fuzzy_match_string_multi(TODO, tags)).collect());
    // TODO wildcard

    if let (Some(name), Some(desc), Some(url)) = (name_match, desc_match, url_match) {
        Some(BookmarkMatch {
            name: deconstruct_success(name),
            desc: deconstruct_success(desc),
            url: deconstruct_success(url),
            tags: None, // TODO
        })
    } else {
        None
    }
}

// TODO test what we have so far in here, then in browser (updated frontend also). THEN do tags and
// wildcard

