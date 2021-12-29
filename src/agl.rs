use wcp_macros::map_from_csv;

static GLYPH_LIST: phf::Map<&'static str, &'static str> = map_from_csv!("maps/glyphlist.txt");

pub fn map_glyph_to_code_points(glyph_name: &str) -> Vec<&str> {
    // drop any chars after dot
    let dropped = step1(glyph_name);
    // split into components
    let split: Vec<&str> = step2(dropped);
    // map each component and concatenate the result
    split.into_iter().map(step3).flatten().collect()
}

pub fn map_code_points_to_string(points: &[&str]) -> Option<String> {
    points.iter().copied().map(parse_unicode).collect()
}

fn parse_unicode(input: &str) -> Option<char> {
    u32::from_str_radix(input, 16).ok().and_then(char::from_u32)
}

fn map_agl(component: &str) -> Option<&str> {
    GLYPH_LIST.get(component).copied()
}

fn map_uni(component: &str) -> Option<Vec<&str>> {
    if component.starts_with("uni") && component[3..].len() % 4 == 0 {
        Some(
            (3..component.len())
                .step_by(4)
                .map(|ix| &component[ix..ix + 4])
                .collect(),
        )
    } else {
        component.strip_prefix('u').map(|stripped| vec![stripped])
    }
}

fn step3(component: &str) -> Vec<&str> {
    if let Some(res) = map_agl(component) {
        vec![res]
    } else if let Some(res) = map_uni(component) {
        res
    } else {
        Vec::new()
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

        cmp("Lcommaaccent", vec!["013B"]);
        cmp("uni20AC0308", vec!["20AC", "0308"]);
        cmp("u1040C", vec!["1040C"]);
        cmp("uniD801DC0C", vec![""]);
        cmp("foo", vec![""]);
        cmp(".notdef", vec![""]);

        cmp2("uni013B", "u013B");
    }

    #[test]
    fn test_map() {
        let cmp = |before, after| assert_eq!(map_glyph_to_code_points(before), after);

        cmp(
            "Lcommaaccent_uni20AC0308_u1040C.alternate",
            vec!["013B", "20AC", "0308", "1040C"],
        );

        cmp("foo", vec![""]);
        cmp(".notdef", vec![""]);
    }
}
