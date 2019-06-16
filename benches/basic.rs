#![feature(test)]

#[cfg(test)]
extern crate test;

extern crate unicode_segmentation;

extern crate porter_stemmer;

use std::fs::File;
use std::io::Read;

use test::Bencher;

use unicode_segmentation::UnicodeSegmentation;

use porter_stemmer::stem_tokenized;

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
