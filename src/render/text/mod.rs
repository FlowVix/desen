//! Massively inspired by and based on the work behind [Glyphon](https://github.com/grovesNL/glyphon)
//!
//! Special thanks to System76 for the incredible [cosmic-text](https://github.com/pop-os/cosmic-text) crate

use cosmic_text::fontdb;

pub mod atlas;
pub mod glyph;

/// finds the closest matched font attributes for the input
pub fn find_closest_attrs<'a>(
    db: &fontdb::Database,
    family: cosmic_text::Family<'a>,
    weight: cosmic_text::Weight,
    style: cosmic_text::Style,
    stretch: cosmic_text::Stretch,
) -> cosmic_text::Attrs<'a> {
    db.query(&fontdb::Query {
        families: &[family],
        weight,
        stretch,
        style,
    })
    .and_then(|id| {
        db.face(id).map(|face| {
            cosmic_text::Attrs::new()
                .stretch(face.stretch)
                .weight(face.weight)
                .style(face.style)
        })
    })
    .unwrap_or(
        cosmic_text::Attrs::new()
            .stretch(stretch)
            .weight(weight)
            .style(style),
    )
    .family(family)
}
