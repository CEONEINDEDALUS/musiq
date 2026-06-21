//! Fuzzy, scored search for the track library.
//!
//! Scoring strategy (higher = better match):
//!   - Exact title match: 1000
//!   - Title starts with query: 800
//!   - Title contains query (consecutive): 600
//!   - Fuzzy title match (all chars in order): 300
//!   - All chars present anywhere (order-independent): 100
//!   - Same bonuses at half weight for artist
//!   - Same bonuses at quarter weight for album / path
//!   - Bonus if the matching section is at a word boundary
//!
//! Ties broken by track sort order (index).

use crate::library::Track;

/// Returns indices from `tracks` sorted by descending relevance score.
/// Only tracks with score > 0 are included (i.e., some match exists).
/// If `query` is empty, returns all indices in order.
pub fn ranked_search<'a>(tracks: &'a [Track], query: &str) -> Vec<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return (0..tracks.len()).collect();
    }

    let mut scored: Vec<(usize, i32)> = tracks
        .iter()
        .enumerate()
        .filter_map(|(i, t)| {
            let score = score_track(t, &q);
            if score > 0 { Some((i, score)) } else { None }
        })
        .collect();

    // Sort by score descending; stable sort keeps original order for equal scores
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.into_iter().map(|(i, _)| i).collect()
}

fn score_track(t: &Track, q: &str) -> i32 {
    score_field(&t.title.to_lowercase(), q, 10)
        + score_field(&t.artist.to_lowercase(), q, 5)
        + score_field(&t.album.to_lowercase(), q, 3)
        + score_field(&t.path.to_string_lossy().to_lowercase(), q, 1)
}

/// Score a single field at given weight multiplier.
fn score_field(field: &str, q: &str, weight: i32) -> i32 {
    if field.is_empty() { return 0; }

    // Exact match
    if field == q {
        return 1000 * weight;
    }
    // Starts-with
    if field.starts_with(q) {
        return 800 * weight + word_boundary_bonus(field, 0, weight);
    }
    // Contains (consecutive substring)
    if let Some(pos) = field.find(q) {
        let base = 600 * weight;
        return base + word_boundary_bonus(field, pos, weight);
    }
    // Fuzzy: all chars of q appear in field in order
    if fuzzy_match(field, q) {
        return 300 * weight;
    }
    // All chars present anywhere (order-independent) — good for progressive typing
    if q.chars().all(|c| field.contains(c)) {
        return 100 * weight;
    }
    0
}

/// Bonus if the match starts at a word boundary (after space, '-', '_', '(').
fn word_boundary_bonus(field: &str, pos: usize, weight: i32) -> i32 {
    if pos == 0 { return 0; } // starts_with already gets full credit
    let before = field.as_bytes().get(pos.saturating_sub(1)).copied().unwrap_or(b' ');
    if matches!(before, b' ' | b'-' | b'_' | b'(' | b'[') {
        50 * weight
    } else {
        0
    }
}

/// Returns true if every character of `q` appears in `field` in order (case-insensitive).
pub fn fuzzy_match(field: &str, q: &str) -> bool {
    let mut fi = field.chars();
    'outer: for qc in q.chars() {
        loop {
            match fi.next() {
                Some(fc) if fc == qc => continue 'outer,
                Some(_) => {}
                None => return false,
            }
        }
    }
    true
}

/// Highlight ranges: returns list of (start_byte, end_byte) for matching
/// portions of `field` given query `q`. Used by the UI for search highlights.
#[cfg(test)]
mod tests {
    use super::*;

    fn make_track(title: &str, artist: &str, album: &str, path: &str) -> Track {
        Track {
            path: std::path::PathBuf::from(path),
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
            track_number: 1,
            duration_secs: 200,
            format: "MP3".to_string(),
            bitrate: Some(320),
            file_size: 1000,
            album_art: None,
        }
    }

    #[test]
    fn single_char_matches_tracks_with_that_char() {
        let tracks = vec![
            make_track("Kendrick Lamar", "Kendrick Lamar", "DAMN.", "/a/kendrick.mp3"),
            make_track("Hello World", "Unknown", "Album", "/b/hello.mp3"),
        ];
        let results = ranked_search(&tracks, "k");
        assert_eq!(results.len(), 2, "both have 'k': Kendrick title + Unknown artist");
    }

    #[test]
    fn two_char_consecutive_finds_substring() {
        let tracks = vec![
            make_track("Kendrick Lamar", "Kendrick Lamar", "DAMN.", "/a/kendrick.mp3"),
            make_track("Hello World", "Unknown", "Album", "/b/hello.mp3"),
        ];
        let results = ranked_search(&tracks, "ke");
        assert_eq!(results.len(), 1, "Kendrick title starts with 'ke'");
    }

    #[test]
    fn two_char_order_independent_finds_out_of_order_chars() {
        let tracks = vec![
            make_track("The Beatles", "The Beatles", "Abbey Road", "/c/beatles.mp3"),
            make_track("Hello World", "Unknown", "Album", "/b/hello.mp3"),
        ];
        // "the beatles" has 't' (pos 0) and 'e' (pos 2) — order-dependent fuzzy matches 't' then 'e'
        // This works through fuzzy_match (in-order) already
        let results = ranked_search(&tracks, "te");
        assert_eq!(results.len(), 1, "Beatles has 't' then 'e' in order");
    }

    #[test]
    fn order_independent_fallback_works() {
        let tracks = vec![
            make_track("Hello World", "Unknown", "Album", "/b/hello.mp3"),
        ];
        // "hello" has 'l' and 'e' but 'e' comes before 'l', so fuzzy in-order "le" fails
        // but the order-independent fallback should catch it
        let results = ranked_search(&tracks, "le");
        assert_eq!(results.len(), 1, "Hello has both 'l' and 'e' even though out of order");
    }

    #[test]
    fn progressive_typing_narrows_results() {
        let tracks = vec![
            make_track("Kendrick Lamar", "Kendrick Lamar", "DAMN.", "/a/kendrick.mp3"),
            make_track("Kanye West", "Kanye West", "Graduation", "/b/kanye.mp3"),
            make_track("The Beatles", "The Beatles", "Abbey Road", "/c/beatles.mp3"),
        ];
        let single = ranked_search(&tracks, "k");
        assert_eq!(single.len(), 2, "Kendrick + Kanye have 'k'");

        let two = ranked_search(&tracks, "ke");
        assert_eq!(two.len(), 2, "both have 'k...e' in order");

        // "kend" only matches Kendrick (starts_with) — Kanye "kanye" doesn't contains "kend"
        let three = ranked_search(&tracks, "kend");
        assert_eq!(three.len(), 1, "only Kendrick has 'kend'");
        assert_eq!(three[0], 0);
    }

}

pub fn highlight_ranges(field: &str, q: &str) -> Vec<(usize, usize)> {
    if q.is_empty() { return Vec::new(); }
    let fl = field.to_lowercase();
    let ql = q.to_lowercase();
    // Prefer consecutive match first
    if let Some(pos) = fl.find(&ql) {
        return vec![(pos, pos + ql.len())];
    }
    // Fall back to highlighting each char independently
    let mut ranges = Vec::new();
    let mut fi = fl.char_indices().peekable();
    for qc in ql.chars() {
        while let Some(&(bi, fc)) = fi.peek() {
            if fc == qc {
                let end = bi + qc.len_utf8();
                ranges.push((bi, end));
                let _ = fi.next();
                break;
            }
            let _ = fi.next();
        }
    }
    ranges
}
