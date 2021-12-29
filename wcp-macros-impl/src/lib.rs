use csv::ReaderBuilder;
use proc_macro2::TokenStream;

pub struct ImplArgs {
    pub path: String,
}

pub fn map_from_csv_impl<A: Into<ImplArgs>>(args: A) -> TokenStream {
    let args = args.into();

    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .comment(Some(b'#'))
        .from_path(&args.path)
        .unwrap();

    let records: Vec<(String, String)> = reader.deserialize().filter_map(|x| x.ok()).collect();

    let mut builder = phf_codegen::Map::new();
    for (key, value) in records {
        let quoted_value = &format!("{:?}", value);
        builder.entry(key, quoted_value);
    }
    let map = builder.build();

    let tokens: TokenStream = map
        .to_string()
        .parse()
        .expect("Could not parse DisplayMap as TokenStream");

    tokens
}
