/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![feature(test)]

#[cfg(test)]
extern crate test;

extern crate unicode_segmentation;

use unicode_segmentation::UnicodeSegmentation;

/// Given a word, return its stemmed form
///
/// # Examples
///
/// ```
/// use porter_stemmer::stem;
///
/// let stemmed = stem("totally");
/// assert_eq!("total", &stemmed);
/// ```
pub fn stem(word: &str) -> String {
    stem_tokenized(word.graphemes(true).collect::<Vec<&str>>()).iter()
        .fold(String::new(), |prev, next| { format!("{}{}", prev, next) })
}

/// Take a word as a Vector of grapheme clusters, and return the stemmed equivalent using Porter's
/// stemming algorithm.
///
/// # Examples
///
/// ```
/// use porter_stemmer::stem_tokenized;
///
/// let tokenized = vec!["s", "t", "e", "m", "m", "i", "n", "g"];
/// let stemmed = stem_tokenized(tokenized);
/// assert_eq!(&["s", "t", "e", "m"], &stemmed[..]);
/// ```
pub fn stem_tokenized(word: Vec<&str>) -> Vec<&str> {
    if word.len() > 2 {
        let word = phase_one_a(word);
        let word = phase_one_b(word);
        let word = phase_one_c(word);
        let word = phase_two(word);
        let word = phase_three(word);
        let word = phase_four(word);
        let word = phase_5a(word);
        let word = phase_5b(word);
        word
    } else {
        word
    }
}

fn real_vowel(grapheme: &str) -> bool {
    match grapheme {
        "a" |
        "e" |
        "i" |
        "o" |
        "u" => {
            true
        },
        _ => false
    }
}

fn real_consonant(grapheme: &str) -> bool {
    !real_vowel(grapheme)
}

fn porter_vowel(word: &[&str], index: usize) -> bool {
    let grapheme = word[index];

    if real_vowel(grapheme) {
        true
    } else {
        if index == 0 || grapheme != "y" {
            false
        } else {
            let preceeding_grapheme = word[index - 1];
            real_consonant(preceeding_grapheme)
        }
    }
}

fn porter_consonant(word: &[&str], index: usize) -> bool {
    !porter_vowel(word, index)
}

fn contains_porter_vowel(word: &[&str]) -> bool {
    for index in 0..word.len() {
        if porter_vowel(word, index) {
            return true;
        }
    }

    return false;
}

fn ends_double_porters_consonant(word: &[&str]) -> bool {
    let word_length = word.len();
    if word_length > 2 {
        let last_grapheme = word[word_length - 1];
        let penultimate_grapheme = word[word_length - 2];

        last_grapheme == penultimate_grapheme &&
            porter_consonant(word, word_length - 1)
    } else {
        false
    }

}

// Condition: *o  the stem ends consonant-vowel-consonant,
// where the second consonant is not w, x or y.
fn ends_star_o(word: &[&str]) -> bool {
    let word_length = word.len();

    if word_length > 2 {
        let last_grapheme = word[word_length - 1];
        match last_grapheme {
            "w" | "x" | "y" => false,
            _ => {
                porter_consonant(word, word_length - 1) &&
                porter_vowel(word, word_length - 2) &&
                porter_consonant(word, word_length - 3)
            }
        }
    } else {
        false
    }
}

/// The Porter stemmer makes use of a _measure_.
///
/// Defined formally as the number of
/// Vowel sequence-Consonant sequence pairs in a word or fragment.
///
/// If C is a sequence of consonants, and V a sequence
/// of vowels, the measure of a word or word part can be
/// defined by:
///
/// C?(VC)*V?
///
/// Where the measure, _m_, is equal to the number of matches
/// by the Kleene star `(VC)*`
///
/// Note how the parameter is a &[&'a str] (slice).  This is so we can use an
/// indexable list of grapheme clusters.
///
/// TODO: Maybe parameterise over Index trait so we can optimise
/// for known single char byte sequences in English? What if
/// the English input has a name with a diacritic?
fn measure(word: &[&str]) -> usize {
    let mut measure = 0;
    let word_length = word.len();

    if word_length == 0{
        return measure;
    }

    let mut is_vowel_current = real_vowel(word[0]);

    for index in 1..word_length {
        let is_vowel = porter_vowel(word, index);
        if !is_vowel_current && is_vowel {
            is_vowel_current = true;
        } else if is_vowel_current && !is_vowel {
            is_vowel_current = false;
            measure += 1;
        }
    }

    return measure;
}

/// Order in which to apply rules:
///
/// SSES -> SS
/// IES  -> I
/// SS -> SS
/// S  ->
fn phase_one_a(word: Vec<&str>) -> Vec<&str> {
    // Move `word` in here where we can make mutable where necessary
    let word_length = word.len();

    if word.ends_with(&["s", "s", "e", "s"]) || word.ends_with(&["i", "e", "s"]) {
        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["s", "s"]) {
        word
    } else if word.ends_with(&["s"]) {
        let mut word = word;
        word.truncate(word_length - 1);
        word
    } else {
        word
    }
}

/// Order in which to apply rules:
///
/// measure > 0 ? EED -> EE
/// *v*         ? ED ->
/// *v*         ? ING ->
fn phase_one_b(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();

    if word.ends_with(&["e", "e", "d"]) {
        if measure(&word[..word_length - 3]) > 0 {
            let mut word = word;
            word.truncate(word_length - 1);
            word
        } else {
            word
        }
    } else if word.ends_with(&["e", "d"]) {
        if contains_porter_vowel(&word[..word_length - 2]) {
            let mut word = word;
            word.truncate(word_length - 2);
            phase_one_b_substep(word)
        } else {
            word
        }
    } else if word.ends_with(&["i", "n", "g"]) {
        if contains_porter_vowel(&word[..word_length - 3]) {
            let mut word = word;
            word.truncate(word_length - 3);
            phase_one_b_substep(word)
        } else {
            word
        }
    } else {
        word
    }
}

///
/// AT -> ATE
/// BL -> BLE
/// IZ -> IZE
///
/// Contraints apply to whole word in here
/// *d (double consonant) and not (*L or *S or *Z) -> change to single letter
///
/// m=1 and *o (see `ends_star_o`) -> E
fn phase_one_b_substep(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();
    if word.ends_with(&["a", "t"]) ||
       word.ends_with(&["b", "l"]) ||
       word.ends_with(&["i", "z"]) {
        let mut word = word;
        word.push("e");
        word
    } else if ends_double_porters_consonant(&word) &&
              !(word.ends_with(&["l"]) ||
                word.ends_with(&["s"]) ||
                word.ends_with(&["z"])) {

        let mut word = word;
        word.truncate(word_length - 1);
        word

    } else if measure(&word) == 1 && ends_star_o(&word) {
        let mut word = word;
        word.push("e");
        word
    } else {
        word
    }
}

///
/// TODO: Question about "contains* vowel and the 'Y' case (see ignored test on sky)
/// *v* Y -> I
fn phase_one_c(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();
    if contains_porter_vowel(&word) && word.ends_with(&["y"]) {
        let mut word = word;
        word[word_length - 1] = "i";
        word
    } else {
        word
    }
}

/// For all where the STEM is measure > 0
/// ATIONAL -> ATE
/// TIONAL  -> TION
/// ENCI    -> ENCE
/// ANCI    -> ANCE
/// IZER    -> IZE
/// ABLI    -> ABLE
/// ALLI    -> AL
/// ENTLI   -> ENT
/// ELI     -> E
/// OUSLI   -> OUS
/// IZATION -> IZE
/// ATION   -> ATE
/// ATOR    -> ATE
/// ALISM   -> AL
/// IVENESS -> IVE
/// FULNESS -> FUL
/// OUSNESS -> OUS
/// ALITI   -> AL
/// IVITI   -> IVE
/// BILITI  -> BLE
// TODO: This is a naive implementation - we can definitely be more efficient here by traversing
// backwards and splitting on the last grapheme rather than searching everything (use a trie to
// hold the search space)
fn phase_two(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();
    if word.ends_with(&["a", "t", "i", "o", "n", "a", "l"]) &&
        measure(&word[..word_length - 7]) > 0 {

        let mut word = word;
        word.truncate(word_length - 5);
        word.push("e");
        word
    } else if word.ends_with(&["t", "i", "o", "n", "a", "l"]) &&
        measure(&word[..word_length - 6]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["e", "n", "c", "i"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word[word_length - 1] = "e";
        word
    } else if word.ends_with(&["a", "n", "c", "i"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word[word_length - 1] = "e";
        word
    } else if word.ends_with(&["i", "z", "e", "r"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word.truncate(word_length - 1);
        word
    } else if word.ends_with(&["a", "b", "l", "i"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word[word_length - 1] = "e";
        word
    } else if word.ends_with(&["a", "l", "l", "i"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["e", "n", "t", "l", "i"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["e", "l", "i"]) &&
        measure(&word[..word_length - 3]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["o", "u", "s", "l", "i"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["i", "z", "a", "t", "i", "o", "n"]) &&
        measure(&word[..word_length - 7]) > 0 {

        let mut word = word;
        word.truncate(word_length - 5);
        word.push("e");
        word
    } else if word.ends_with(&["a", "t", "i", "o", "n"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word.push("e");
        word
    } else if word.ends_with(&["a", "t", "o", "r"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word.push("e");
        word
    } else if word.ends_with(&["a", "l", "i", "s", "m"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "v", "e", "n", "e", "s", "s"]) &&
        measure(&word[..word_length - 7]) > 0 {

        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["f", "u", "l", "n", "e", "s", "s"]) &&
        measure(&word[..word_length - 7]) > 0 {

        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["o", "u", "s", "n", "e", "s", "s"]) &&
        measure(&word[..word_length - 7]) > 0 {

        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["a", "l", "i", "t", "i"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "v", "i", "t", "i"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word.push("e");
        word
    } else if word.ends_with(&["b", "i", "l", "i", "t", "i"]) &&
        measure(&word[..word_length - 6]) > 0 {

        let mut word = word;
        word.truncate(word_length - 5);
        word.push("l");
        word.push("e");
        word
    } else {
        word
    }
}

/// For all whre the STEM measure is greater than one
/// ICATE -> IC
/// ATIVE ->
/// ALIZE -> AL
/// ICITI -> IC
/// ICAL  -> IC
/// FUL   ->
/// NESS  ->
// TODO: see phase_two
fn phase_three(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();
    if word.ends_with(&["i", "c", "a", "t", "e"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["a", "t", "i", "v", "e"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 5);
        word
    } else if word.ends_with(&["a", "l", "i", "z", "e"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "c", "i", "t", "i"]) &&
        measure(&word[..word_length - 5]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "c", "a", "l"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["f", "u", "l"]) &&
        measure(&word[..word_length - 3]) > 0 {

        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["n", "e", "s", "s"]) &&
        measure(&word[..word_length - 4]) > 0 {

        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else {
        word
    }
}

fn phase_four(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();
    if word.ends_with(&["a", "l"]) &&
        measure(&word[..word_length - 2]) > 1 {
        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["a", "n", "c", "e"]) &&
        measure(&word[..word_length - 4]) > 1 {
        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["e", "n", "c", "e"]) &&
        measure(&word[..word_length - 4]) > 1 {
        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["e", "r"]) &&
        measure(&word[..word_length - 2]) > 1 {
        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["i", "c"]) &&
        measure(&word[..word_length - 2]) > 1 {
        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["a", "b", "l", "e"]) &&
        measure(&word[..word_length - 4]) > 1 {
        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["i", "b", "l", "e"]) &&
        measure(&word[..word_length - 4]) > 1 {
        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["a", "n", "t"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["e", "m", "e", "n", "t"]) &&
        measure(&word[..word_length - 5]) > 1 {
        let mut word = word;
        word.truncate(word_length - 5);
        word
    } else if word.ends_with(&["m", "e", "n", "t"]) &&
        measure(&word[..word_length - 4]) > 1 {
        let mut word = word;
        word.truncate(word_length - 4);
        word
    } else if word.ends_with(&["e", "n", "t"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "o", "n"]) &&
        measure(&word[..word_length - 3]) > 1 {

        let last_grapheme_in_stem = word[word_length - 4];
        if last_grapheme_in_stem == "s" || last_grapheme_in_stem == "t" {
            let mut word = word;
            word.truncate(word_length - 3);
            word
        } else {
            word
        }
    } else if word.ends_with(&["o", "u"]) &&
        measure(&word[..word_length - 2]) > 1 {
        let mut word = word;
        word.truncate(word_length - 2);
        word
    } else if word.ends_with(&["i", "s", "m"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["a", "t", "e"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "t", "i"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["o", "u", "s"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "v", "e"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else if word.ends_with(&["i", "z", "e"]) &&
        measure(&word[..word_length - 3]) > 1 {
        let mut word = word;
        word.truncate(word_length - 3);
        word
    } else {
        word
    }

}

fn phase_5a(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();

    if word.ends_with(&["e"]) &&
        measure(&word[..word_length - 1]) > 1 {
        let mut word = word;
        word.truncate(word_length - 1);
        word
    } else if word.ends_with(&["e"]) &&
        measure(&word[..word_length - 1]) == 1 &&
        !ends_star_o(&word[..word_length - 1]) {

        let mut word = word;
        word.truncate(word_length - 1);
        word
    } else {
        word
    }
}

fn phase_5b(word: Vec<&str>) -> Vec<&str> {
    let word_length = word.len();
    if word.ends_with(&["l"]) &&
        measure(&word) > 1 &&
        ends_double_porters_consonant(&word) {

        let mut word = word;
        word.truncate(word_length - 1);
        word
    } else {
        word
    }
}
#[test]
fn test_real_vowel() {
    assert!(real_vowel("a"));
    assert!(real_vowel("e"));
    assert!(real_vowel("i"));
    assert!(real_vowel("o"));
    assert!(real_vowel("u"));
    assert!(!real_vowel("b"));
}

#[test]
fn test_real_consonant() {
    assert!(!real_consonant("a"));
    assert!(!real_consonant("e"));
    assert!(!real_consonant("i"));
    assert!(!real_consonant("o"));
    assert!(!real_consonant("u"));
    assert!(real_consonant("b"));
}

#[cfg(test)]
fn tokenise<'a>(input: &'a str) -> Vec<&'a str> {
    use unicode_segmentation::UnicodeSegmentation;
    input.graphemes(true).collect::<Vec<&'a str>>()
}

#[cfg(test)]
fn assert_fn<'a>(f: fn(Vec<&'a str>) -> Vec<&'a str>, input: &'a str, expected: &'a str) {
    let input = tokenise(input);
    let expected = tokenise(expected);

    assert_eq!(&f(input), &expected);
}


#[test]
fn test_porter_character_types() {
    let graphemes = tokenise("toy");

    assert!(porter_consonant(&graphemes, 0));
    assert!(porter_vowel(&graphemes, 1));
    assert!(porter_consonant(&graphemes, 2));

    let graphemes = tokenise("syzygy");
    assert!(porter_consonant(&graphemes, 0));
    assert!(porter_vowel(&graphemes, 1));
    assert!(porter_consonant(&graphemes, 2));
    assert!(porter_vowel(&graphemes, 3));
    assert!(porter_consonant(&graphemes, 4));
    assert!(porter_vowel(&graphemes, 5));
}

#[test]
fn test_ends_double_porters_consonant() {
   let graphemes = tokenise("sell");
   assert!(ends_double_porters_consonant(&graphemes));

   let graphemes = tokenise("greyy");
   assert!(!ends_double_porters_consonant(&graphemes));

   let graphemes = tokenise("see");
   assert!(!ends_double_porters_consonant(&graphemes));
}

#[test]
fn test_contains_vowel() {
    let graphemes = tokenise("toy");
    assert!(contains_porter_vowel(&graphemes));

    let graphemes = tokenise("syzygy");
    assert!(contains_porter_vowel(&graphemes));

    let graphemes = tokenise("trjk");
    assert!(!contains_porter_vowel(&graphemes));
}

#[test]
fn test_ends_star_o() {
    let graphemes = tokenise("awhil");
    assert!(ends_star_o(&graphemes));

    let graphemes = tokenise("mix");
    assert!(!ends_star_o(&graphemes));

    let graphemes = tokenise("dew");
    assert!(!ends_star_o(&graphemes));

    let graphemes = tokenise("day");
    assert!(!ends_star_o(&graphemes));
}

#[test]
fn test_measure() {
    let graphemes = tokenise("crepuscular");
    assert_eq!(4, measure(&graphemes[..]));

    let graphemes = tokenise("bacon");
    assert_eq!(2, measure(&graphemes[..]));

    let graphemes = tokenise("abacus");
    assert_eq!(3, measure(&graphemes[..]));


    let graphemes = tokenise("paackkeeer");
    assert_eq!(2, measure(&graphemes[..]));

    let graphemes = tokenise("syzygy");
    assert_eq!(2, measure(&graphemes[..]));

}

#[test]
fn test_phase_one() {
    assert_fn(phase_one_a, "caresses", "caress");
    assert_fn(phase_one_a, "caress", "caress");
    assert_fn(phase_one_a, "ponies", "poni");
    assert_fn(phase_one_a, "cats", "cat");
}

#[test]
fn test_phase_one_b() {
    assert_fn(phase_one_b, "feed", "feed");
    assert_fn(phase_one_b, "agreed", "agree");
    assert_fn(phase_one_b, "plastered", "plaster");
    assert_fn(phase_one_b, "bled", "bled");
    assert_fn(phase_one_b, "motoring", "motor");
    assert_fn(phase_one_b, "sing", "sing");
}

#[test]
fn test_phase_one_b_substep() {
    assert_fn(phase_one_b_substep, "conflat", "conflate");
    assert_fn(phase_one_b_substep, "troubl", "trouble");
    assert_fn(phase_one_b_substep, "siz", "size");
    assert_fn(phase_one_b_substep, "hopp", "hop");
    assert_fn(phase_one_b_substep, "hiss", "hiss");
    assert_fn(phase_one_b_substep, "fizz", "fizz");
    assert_fn(phase_one_b_substep, "fall", "fall");
    assert_fn(phase_one_b_substep, "fail", "fail");
    assert_fn(phase_one_b_substep, "fil", "file");
}

#[test]
fn test_phase_one_c() {
    assert_fn(phase_one_c, "happy", "happi");
}

#[test]
#[ignore]
fn test_phase_one_c_sky() {
    assert_fn(phase_one_c, "sky", "sky");
}

#[test]
fn test_phase_two() {
    assert_fn(phase_two, "relational", "relate");
    assert_fn(phase_two, "conditional", "condition");
    assert_fn(phase_two, "rational", "rational");
    assert_fn(phase_two, "valenci", "valence");
    assert_fn(phase_two, "hesitanci", "hesitance");
    assert_fn(phase_two, "digitizer", "digitize");
    assert_fn(phase_two, "conformabli", "conformable");
    assert_fn(phase_two, "radicalli", "radical");
    assert_fn(phase_two, "differentli", "different");
    assert_fn(phase_two, "vileli", "vile");
    assert_fn(phase_two, "analogousli", "analogous");
    assert_fn(phase_two, "vietnamization", "vietnamize");
    assert_fn(phase_two, "predication", "predicate");
    assert_fn(phase_two, "operator", "operate");
    assert_fn(phase_two, "feudalism", "feudal");
    assert_fn(phase_two, "decisiveness", "decisive");
    assert_fn(phase_two, "hopefulness", "hopeful");
    assert_fn(phase_two, "callousness", "callous");
    assert_fn(phase_two, "formaliti", "formal");
    assert_fn(phase_two, "sensitiviti", "sensitive");
    assert_fn(phase_two, "sensibiliti", "sensible");
}

#[test]
fn test_phase_three() {
    assert_fn(phase_three, "triplicate", "triplic");
    assert_fn(phase_three, "formative", "form");
    assert_fn(phase_three, "formalize", "formal");
    assert_fn(phase_three, "electriciti", "electric");
    assert_fn(phase_three, "electrical", "electric");
    assert_fn(phase_three, "hopeful", "hope");
    assert_fn(phase_three, "goodness", "good");
}

#[test]
fn test_phase_four() {
    assert_fn(phase_four, "revival", "reviv");
    assert_fn(phase_four, "allowance", "allow");
    assert_fn(phase_four, "inference", "infer");
    assert_fn(phase_four, "airliner", "airlin");
    assert_fn(phase_four, "gyroscopic", "gyroscop");
    assert_fn(phase_four, "adjustable", "adjust");
    assert_fn(phase_four, "defensible", "defens");
    assert_fn(phase_four, "irritant", "irrit");
    assert_fn(phase_four, "replacement", "replac");
    assert_fn(phase_four, "adjustment", "adjust");
    assert_fn(phase_four, "dependent", "depend");
    assert_fn(phase_four, "adoption", "adopt");
    assert_fn(phase_four, "homologou", "homolog");
    assert_fn(phase_four, "communism", "commun");
    assert_fn(phase_four, "activate", "activ");
    assert_fn(phase_four, "angulariti", "angular");
    assert_fn(phase_four, "homologous", "homolog");
    assert_fn(phase_four, "effective", "effect");
    assert_fn(phase_four, "bowdlerize", "bowdler");
}

#[test]
fn test_phase_five_a() {
    // 5a
    assert_fn(phase_5a, "probate", "probat");
    assert_fn(phase_5a, "rate", "rate");
    assert_fn(phase_5a, "cease", "ceas");
}

#[test]
fn test_phase_five_b() {
    // 5b
    assert_fn(phase_5b, "controll", "control");
    assert_fn(phase_5b, "roll", "roll");
}

#[test]
fn test_stem_tokenized() {
    assert_fn(stem_tokenized, "surveillance", "surveil");
}

#[cfg(test)]
mod test_bench {
    use std::fs::File;
    use std::io::Read;
    use super::*;
    use test::Bencher;
    use unicode_segmentation::UnicodeSegmentation;

    #[bench]
    fn bench_stem(b: &mut Bencher) {
        let mut input     = File::open("input.txt").unwrap();
        let mut expected  = File::open("expected.txt").unwrap();

        let mut input_s = String::new();
        input.read_to_string(&mut input_s).unwrap();

        let mut expected_s = String::new();
        expected.read_to_string(&mut expected_s).unwrap();

        let input = input_s.graphemes(true).collect::<Vec<&str>>();
        let expected = input_s.graphemes(true).collect::<Vec<&str>>();

        b.iter(|| {
            let input = input.clone();
            assert_eq!(expected, stem_tokenized(input));
        });
    }
}
