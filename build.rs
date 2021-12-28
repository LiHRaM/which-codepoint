use csv::ReaderBuilder;
use proc_macro2::TokenStream;
use proc_quote::quote;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    create_maps()?;

    Ok(())
}

fn create_maps() -> anyhow::Result<()> {
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

    let map_tokens: TokenStream = map.to_string().parse().unwrap();

    let output = quote! {
        static GLYPH_LIST: phf::Map<&'static str, &'static str> = #map_tokens;
    };

    let output = output.to_string();

    let path = Path::new(&env::var("OUT_DIR")?).join("glyphlist.rs");
    let mut file = BufWriter::new(File::create(&path)?);
    writeln!(&mut file, "{}", output)?;

    Ok(())
}
