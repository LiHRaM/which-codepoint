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
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Options)]
struct Cli {
    #[options(help = "print help message")]
    help: bool,

    #[options(free, required, help = "path to font")]
    path: String,
}

fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Parsing args");
    let cli = Cli::parse_args_default_or_exit();

    dump_info(cli)
}

fn dump_info(cli: Cli) -> anyhow::Result<()> {
    info!("Load into buffer");
    let buffer = std::fs::read(&cli.path)?;

    info!("Setting ReadScope");
    let scope = ReadScope::new(&buffer);

    info!("Parse file");
    let font_file = scope.read::<FontData>()?;

    info!("Get table provider");
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
    for glyph_id in 0..maxp.num_glyphs {
        let name = names.glyph_name(glyph_id);
        let points = map_glyph_to_code_points(&name);
        let string = map_code_points_to_string(&points).unwrap_or_default();
        println!("{}: {} - {:?} - {}", glyph_id, name, points, string);
    }

    Ok(())
}
