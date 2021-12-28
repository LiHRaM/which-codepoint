use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use csv::ReaderBuilder;

fn main() -> anyhow::Result<()> {
    let path = Path::new(&env::var("OUT_DIR")?).join("glyphlist.rs");
    let mut file = BufWriter::new(File::create(&path)?);

    write!(
        &mut file,
        "static GLYPH_LIST: phf::Map<&'static str, &'static str> = "
    )?;

    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .comment(Some(b'#'))
        .from_path("maps/glyphlist.txt")?;

    let records: Vec<(String, String)> = reader.deserialize().filter_map(|x| x.ok()).collect();

    let mut builder = phf_codegen::Map::new();
    for (key, value) in records {
        builder.entry(key, &format!(r#""{}""#, value));
    }
    let map = builder.build();

    write!(&mut file, "{}", map)?;
    writeln!(&mut file, ";")?;

    Ok(())
}
