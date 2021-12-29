mod agl;

use agl::map_glyph_to_code_points;
use allsorts::{
    binary::read::ReadScope,
    font::{read_cmap_subtable, Encoding},
    font_data::FontData,
    glyph_info::GlyphNames,
    tables::{cmap::Cmap, FontTableProvider, MaxpTable, NameTable},
    tag,
};
use encoding_rs::{MACINTOSH, UTF_16BE};
use gumdrop::Options;
use json::object;
use std::{convert, fmt::Debug};
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Debug, Options)]
struct Cli {
    #[options(help = "print help message")]
    help: bool,

    #[options(free, required, help = "path to font")]
    path: String,
}

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_env("LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("error"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse_args_default_or_exit();

    dump_codepoints(cli)
}

fn decode(encoding: &'static encoding_rs::Encoding, data: &[u8]) -> String {
    let mut decoder = encoding.new_decoder();
    if let Some(size) = decoder.max_utf8_buffer_length(data.len()) {
        let mut s = String::with_capacity(size);
        let (_res, _read, _repl) = decoder.decode_to_string(data, &mut s, true);
        s
    } else {
        String::new() // can only happen if buffer is enormous
    }
}

fn dump_codepoints(cli: Cli) -> anyhow::Result<()> {
    let buffer = std::fs::read(&cli.path)?;
    let scope = ReadScope::new(&buffer);
    let font_file = scope.read::<FontData>()?;
    let table_provider = font_file.table_provider(0)?;

    let table = table_provider
        .table_data(tag::NAME)?
        .expect("no name table");
    let scope = ReadScope::new(&table);
    let name_table = scope.read::<NameTable>()?;

    let full_font_name: &str = &name_table
        .name_records
        .iter()
        .find_map(|rec| {
            if rec.name_id == 4 {
                let platform = rec.platform_id;
                let encoding = rec.encoding_id;
                let language = rec.language_id;
                let offset = usize::from(rec.offset);
                let length = usize::from(rec.length);
                let name_data = name_table
                    .string_storage
                    .offset_length(offset, length)
                    .ok()?
                    .data();
                let name = match (platform, encoding, language) {
                    (0, _, _) => decode(UTF_16BE, name_data),
                    (1, 0, _) => decode(MACINTOSH, name_data),
                    (3, 0, _) => decode(UTF_16BE, name_data),
                    (3, 1, _) => decode(UTF_16BE, name_data),
                    (3, 10, _) => decode(UTF_16BE, name_data),
                    _ => format!(
                        "(unknown platform={} encoding={} language={})",
                        platform, encoding, language
                    ),
                };
                Some(name)
            } else {
                None
            }
        })
        .unwrap_or_default();

    let table = table_provider
        .table_data(tag::MAXP)?
        .expect("no maxp table");
    let scope = ReadScope::new(&table);
    let maxp = scope.read::<MaxpTable>()?;

    let post_data = table_provider
        .table_data(tag::POST)
        .ok()
        .and_then(convert::identity)
        .map(|data| Box::from(&*data));

    let table = table_provider.table_data(tag::CMAP)?;
    let scope = table.as_ref().map(|data| ReadScope::new(data));
    let cmap = scope.map(|scope| scope.read::<Cmap<'_>>()).transpose()?;
    let cmap_subtable = cmap
        .as_ref()
        .and_then(|cmap| read_cmap_subtable(cmap).ok())
        .and_then(convert::identity);

    if !matches!(cmap_subtable, Some((Encoding::Unicode, _))) {
        tracing::error!("CMAP is not unicode");
        std::process::exit(1);
    }

    let names = GlyphNames::new(&cmap_subtable, post_data);
    let mut dropped_points = 0usize;

    (0..maxp.num_glyphs)
        .map(|glyph_id| {
            let name: &str = &names.glyph_name(glyph_id);
            let points = map_glyph_to_code_points(name);
            if points.is_empty() {
                dropped_points += 1;
            }
            object! {
                font_name: full_font_name,
                unicode: points,
                name: name,
            }
        })
        .for_each(|obj| println!("{}", obj));

    info!("Dropped points: {}", dropped_points);

    Ok(())
}
