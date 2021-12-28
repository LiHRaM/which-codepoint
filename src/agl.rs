include!(concat!(env!("OUT_DIR"), "/glyphlist.rs"));

pub fn map_glyph_to_string(glyph_name: &str) -> String {
    // drop any chars after dot
    let dropped = step1(glyph_name);
    // split into components
    let split: Vec<&str> = step2(dropped);
    // map each component and concatenate the result
    split.into_iter().map(step3).collect()
}

fn map_agl(component: &str) -> Option<String> {
    GLYPH_LIST.get(component).map(|c| c.to_string())
}

fn map_uni(component: &str) -> Option<String> {
    if component.starts_with("uni") && component[3..].len() % 4 == 0 {
        (3..component.len())
            .step_by(4)
            .map(|ix| &component[ix..ix + 4])
            .map(unicode)
            .collect::<Option<String>>()
    } else if let Some(stripped) = component.strip_prefix('u') {
        unicode(stripped).map(|c| c.to_string())
    } else {
        None
    }
}

fn unicode(input: &str) -> Option<char> {
    u32::from_str_radix(input, 16).ok().and_then(char::from_u32)
}

fn step3(component: &str) -> String {
    if let Some(res) = map_agl(component) {
        res
    } else if let Some(res) = map_uni(component) {
        res
    } else {
        "".into()
    }
}

fn step2(dropped: &str) -> Vec<&str> {
    dropped.split('\u{005F}').collect()
}

fn step1(glyph_name: &str) -> &str {
    glyph_name.split('\u{002E}').next().unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step1() {
        let cmp = |b, a| assert_eq!(step1(b), a);

        cmp(".notdef", "");
        cmp("test", "test");
    }

    #[test]
    fn test_step2() {
        let cmp = |b, a| assert_eq!(step2(b), a);
        cmp(
            "Lcommaaccent_uni20AC0308_u1040C",
            vec!["Lcommaaccent", "uni20AC0308", "u1040C"],
        );
    }

    #[test]
    fn test_step3() {
        let cmp = |b, a| assert_eq!(step3(b), a);
        let cmp2 = |b, a| assert_eq!(step3(b), step3(a));

        cmp("Lcommaaccent", "\u{013B}");
        cmp("uni20AC0308", "\u{20AC}\u{0308}");
        cmp("u1040C", "\u{1040C}");
        cmp("uniD801DC0C", "");
        cmp("foo", "");
        cmp(".notdef", "");

        cmp2("uni013B", "u013B");
    }

    #[test]
    fn test_map() {
        let cmp = |before: &str, after: &str| assert_eq!(map_glyph_to_string(before), after);

        cmp(
            "Lcommaaccent_uni20AC0308_u1040C.alternate",
            "\u{013B}\u{20AC}\u{0308}\u{1040C}",
        );

        cmp("foo", "");
        cmp(".notdef", "");
    }
}
