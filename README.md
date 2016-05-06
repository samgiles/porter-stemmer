# Porter Stemmer

An implementation of [the Porter stemming algorithm](http://snowball.tartarus.org/algorithms/porter/stemmer.html) in Rust. It operates over
grapheme clusters rather than characters, so your input stream can mixed
content.

# Example

```
use unicode_segmentation::UnicodeSegmentation;
use porter_stemmer::stem;


    let original = "Almost  forty  years  later,  these  fair information  practices  have  become  the standard  for  privacy  protection  around  the  world.  And  yet,  over  that same time  period,  we  have  seen  an  exponential  growth  in  the  use  of surveillance technologies,  and  our  daily  interactions  are  now  routinely  captured, recorded, and manipulated by small and large institutions alike.";

    let tokenised_sentence = original.clone().unicode_words();

    println!("Original:\n{}", original);
    println!("Stemmed:\n{}", tokenised_sentence.map(stem).fold(String::new(), |last, next| { format!("{}{} ", last, next)}));

```
```
Original:
Almost  forty  years  later,  these  fair information  practices  have  become
the standard  for  privacy  protection  around  the  world.  And  yet,  over
that same time  period,  we  have  seen  an  exponential  growth  in  the  use
of surveillance technologies,  and  our  daily  interactions  are  now
routinely  captured, recorded, and manipulated by small and large institutions
alike.

Stemmed:
Almost forti year later these fair inform practic have becom the standard for
privaci protect around the world And yet over that same time period we have
seen an exponenti growth in the us of surveil technologi and our daili interact
ar now routin captur record and manipul by small and larg institut alik
```

Passage of text from [Lessons from the Identity Trail](http://idtrail.org/content/view/799) used only as an example - License: https://creativecommons.org/licenses/by-nc-nd/2.5/ca/

# License

MPL-2.0
