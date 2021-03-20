use std::collections::HashMap;

pub fn insert_tag(seq: &str, ins: &str) -> String {
    let insert = ins.to_uppercase();
    let trans = translate_dna(&insert);
    check_tag(&insert);
    seq.replace("*", &trans).to_uppercase()
}

fn check_tag(insert: &str) {
    insert.chars()
        .for_each(| dna | 
            match dna {
                'A' | 'G' | 'T' | 'C' => (),
                _ => panic!("INVALID TAG DNA SEQUENCES")
            }
        )
}

fn translate_dna(insert: &str) -> String {
    let libs = get_dna_libs();
    let dna = insert.to_uppercase();

    let mut translate = String::new();

    dna.chars()
        .for_each(|b| {
            let base = libs.get(&b).unwrap();
            translate.push(*base);
        });

    translate
}

fn get_dna_libs() -> HashMap<char, char> {
    let dna = String::from("ATGC");
    let comp = String::from("TACG");

    let mut trans = HashMap::new();

    dna.chars()
        .zip(comp.chars())
        .for_each(|(b, c)| {
            trans.insert(b,c);
        });
    
    trans
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn invalid_tag_test() {
        let tag = "ATGTTABCG";

        check_tag(&tag);
    }

    #[test]
    fn tag_insertion_test() {
        let tag = "ATG";
        let seq = "ATTTGT*C";
        let res = String::from("ATTTGTTACC");

        assert_eq!(res, insert_tag(seq, tag));
    }

    #[test]
    fn tag_insertion_lowercase_test() {
        let tag = "atG";
        let seq = "ATTTGT*C";
        let res = String::from("ATTTGTTACC");

        assert_eq!(res, insert_tag(seq, tag));
    }

    #[test]
    fn translate_dna_test() {
        let dna = "ATGC";
        let res = String::from("TACG");

        assert_eq!(res, translate_dna(dna));
    }
}