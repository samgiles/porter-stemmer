extern crate unicode_segmentation;
extern crate porter_stemmer;

use unicode_segmentation::UnicodeSegmentation;
use porter_stemmer::stem;

fn main() {

    let original = "Almost  forty  years  later,  these  fair information  practices  have  become  the standard  for  privacy  protection  around  the  world.  And  yet,  over  that same time  period,  we  have  seen  an  exponential  growth  in  the  use  of surveillance technologies,  and  our  daily  interactions  are  now  routinely  captured, recorded, and manipulated by small and large institutions alike.";

    let tokenised_sentence = original.clone().unicode_words();

    println!("Original:\n{}", original);
    println!("Stemmed:\n{}", tokenised_sentence.map(stem).fold(String::new(), |last, next| { format!("{}{} ", last, next)}));

}
