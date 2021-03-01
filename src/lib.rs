use bytes::Bytes;
use lazy_static::lazy_static;
use pcre2::bytes::{Regex, RegexBuilder};
use std::str::from_utf8;

mod test_stemmer_uk;

fn ukstemmer_search_preprocess(word: String) -> String {
    word.to_lowercase()
        .replace("'", "")
        .replace("ё", "е")
        .replace("ъ", "ї")
}

lazy_static! {
    // http://uk.wikipedia.org/wiki/Голосний_звук
    static ref VOVEL: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"аеиоуюяіїє").unwrap();
    static ref PERFECTIVEGROUND: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"(ив|ивши|ившись|ыв|ывши|ывшись((?<=[ая])(в|вши|вшись)))$").unwrap();
    //  http://uk.wikipedia.org/wiki/Рефлексивне_дієслово
    static ref REFLEXIVE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"(с[яьи])$").unwrap();
    // http://uk.wikipedia.org/wiki/Прикметник + http://wapedia.mobi/uk/Прикметник
    static ref ADJECTIVE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"(ими|ій|ий|а|е|ова|ове|ів|є|їй|єє|еє|я|ім|ем|им|ім|их|іх|ою|йми|іми|у|ю|ого|ому|ої)$").unwrap();
    // http://uk.wikipedia.org/wiki/Дієприкметник
    static ref PARTICIPLE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"(ий|ого|ому|им|ім|а|ій|у|ою|ій|і|их|йми|их)$").unwrap();
    // http://uk.wikipedia.org/wiki/Дієслово
    static ref VERB: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"(сь|ся|ив|ать|ять|у|ю|ав|али|учи|ячи|вши|ши|е|ме|ати|яти|є)$").unwrap();
    // http://uk.wikipedia.org/wiki/Іменник
    static ref NOUN: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"(а|ев|ов|е|ями|ами|еи|и|ей|ой|ий|й|иям|ям|ием|ем|ам|ом|о|у|ах|иях|ях|ы|ь|ию|ью|ю|ия|ья|я|і|ові|ї|ею|єю|ою|є|еві|ем|єм|ів|їв|ю)$").unwrap();
    // http://uk.wikipedia.org/wiki/Голосний_звук
    static ref RVRE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"[аеиоуюяіїє]").unwrap();
    static ref DERIVATIONAL: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"[^аеиоуюяіїє][аеиоуюяіїє]+[^аеиоуюяіїє]+[аеиоуюяіїє].*(?<=о)сть?$").unwrap();
    static ref N1_RE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"и$").unwrap();
    static ref N2_RE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"ость$").unwrap();
    static ref N3_RE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"ь$").unwrap();
    static ref N4_RE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"ейше?$").unwrap();
    static ref N5_RE: Regex = RegexBuilder::new()
            .utf(true)
            .ucp(true)
            .build(r"нн$").unwrap();
}

fn s<'a>(st: &[u8], reg: &Regex, to: &[u8], rv: &mut Bytes) -> bool {
    let orig = st;
    let res = reg.find(st).unwrap();
    if let Some(m) = res {
        let result = replace(st, to, m.start(), m.end());
        *rv = result;
    }

    !orig.eq(rv)
}

fn replace<'a>(st: &'a [u8], replacer: &'a [u8], start: usize, end: usize) -> Bytes {
    let mut bytes = Bytes::with_capacity(start + replacer.len() + st[end..].len());
    bytes.extend_from_slice(&st[..start]);
    bytes.extend_from_slice(replacer);
    bytes.extend_from_slice(&st[end..]);

    return bytes;
}

#[test]
fn replace_test() {
    let reg = RegexBuilder::new()
        .utf(true)
        .ucp(true)
        .build(r"123")
        .unwrap();
    let s = "012345678".as_bytes();
    let v = reg.find(s).unwrap().unwrap();

    assert_eq!(
        replace(s, "_".as_bytes(), v.start(), v.end()),
        "0_45678".as_bytes()
    );
}

fn as_str(b: &[u8]) -> String {
    from_utf8(b).expect("not correct utf8 bytes").to_string()
}

pub fn stem_word(word: String) -> String {
    let word = ukstemmer_search_preprocess(word.clone());

    if let Ok(Some(m)) = RVRE.find(word.clone().as_bytes()) {
        let m_end = m.end();

        let start = Bytes::from(word.clone().as_bytes()[0..m_end].as_ref());
        let mut rv = Bytes::from(word.clone().as_bytes()[m_end..].as_ref());

        // Step 1
        if !s(&rv.clone()[..], &PERFECTIVEGROUND, "".as_bytes(), &mut rv) {
            s(&rv.clone()[..], &REFLEXIVE, "".as_bytes(), &mut rv);

            if s(&rv.clone()[..], &ADJECTIVE, "".as_bytes(), &mut rv) {
                s(&rv.clone()[..], &PARTICIPLE, "".as_bytes(), &mut rv);
            } else {
                if !s(&rv.clone()[..], &VERB, "".as_bytes(), &mut rv) {
                    s(&rv.clone()[..], &NOUN, "".as_bytes(), &mut rv);
                }
            }
        }
        // Step 2
        s(&rv.clone()[..], &N1_RE, "".as_bytes(), &mut rv);

        // Step 3
        if let Ok(Some(_)) = DERIVATIONAL.find(&rv.clone()[..]) {
            s(&rv.clone()[..], &N2_RE, "".as_bytes(), &mut rv);
        }

        // Step 4
        if s(&rv.clone()[..], &N3_RE, "".as_bytes(), &mut rv) {
            s(&rv.clone()[..], &N4_RE, "".as_bytes(), &mut rv);
            s(&rv.clone()[..], &N5_RE, "н".as_bytes(), &mut rv);
        }
        let mut res = Vec::with_capacity(start.len() + &rv.len());
        res.append(&mut start.to_vec());
        res.append(&mut rv.to_vec());

        as_str(res.as_ref())
    } else {
        word
    }
}

#[test]
fn stem_word_test() {
    assert_eq!(stem_word("ручкається".into()), "ручкаєт",);
}
