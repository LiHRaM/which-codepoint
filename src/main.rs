mod agl;

use agl::{map_code_points_to_string, map_glyph_to_code_points};
use allsorts::{
    binary::read::ReadScope,
    font::{read_cmap_subtable, Encoding},
    font_data::FontData,
    glyph_info::GlyphNames,
    tables::{cmap::Cmap, FontTableProvider, MaxpTable},
    tag,
};
use gumdrop::Options;
use std::{convert, fmt::Debug};
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Debug, Options)]
struct Cli {
    #[options(help = "print help message")]
    help: bool,

    #[options(free, required, help = "path to font")]
    path: String,
}

fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_env("LOG_LEVEL").unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let cli = Cli::parse_args_default_or_exit();

    dump_codepoints(cli)
}

fn dump_codepoints(cli: Cli) -> anyhow::Result<()> {
    let buffer = std::fs::read(&cli.path)?;
    let scope = ReadScope::new(&buffer);
    let font_file = scope.read::<FontData>()?;
    let table_provider = font_file.table_provider(0)?;

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
    for glyph_id in 0..maxp.num_glyphs {
        let name = names.glyph_name(glyph_id);
        let points = map_glyph_to_code_points(&name);
        if points.is_empty() {
            dropped_points += 1;
        }
        let string = map_code_points_to_string(&points).unwrap_or_default();
        debug!("{}: {} - {:?} - {}", glyph_id, name, points, string);
    }

    info!("Dropped points: {}", dropped_points);

    Ok(())
}
