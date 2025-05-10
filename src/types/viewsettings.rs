use chumsky::Parser as ChumskyParser;

use crate::{
    parser::{close_block, key_value, open_block, InternalParser, TokenError, TokenSource},
    Parser,
};

#[derive(Debug, Default)]
pub struct ViewSettings {
    snap_to_grid: bool,
    show_grid: bool,
    show_logical_grid: bool,
    grid_spacing: u32,
    show_3d_grid: bool,
    hide_objects: bool,
    hide_walls: bool,
    hide_stripes: bool,
    hide_neighbors: bool,
    hide_detail: bool,
    show_brushes: bool,
    show_entities: bool,
    show_light_radius: bool,
    show_lighting_preview: bool,
    show_wireframe: bool,
}

/// Public parser trait implementation that allows [`ViewSettings`] to use ::parse(input) call.
impl Parser<'_> for ViewSettings {}

/// A [`ViewSettings`] implementation for [`ViewSettings`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let view_settings = ViewSettings::parser().parse();`.
///
/// The format that is being parsed here is:
/// viewsettings
///{
/// "bSnapToGrid" "1"
/// "bShowGrid" "1"
/// "bShowLogicalGrid" "0"
/// "nGridSpacing" "64"
/// "bShow3DGrid" "0"
///}
impl<'src> InternalParser<'src> for ViewSettings {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        open_block("viewsettings")
            .ignore_then(
                key_value("snap_to_grid")
                    .map(|v| Self {
                        snap_to_grid: v.parse().unwrap(),
                        ..Self::default()
                    })
                    .then(key_value("show_grid"))
                    .map(|(mut vs, sg)| {
                        vs.show_grid = sg.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_logical_grid"))
                    .map(|(mut vs, slg)| {
                        vs.show_logical_grid = slg.parse().unwrap();
                        vs
                    })
                    .then(key_value("grid_spacing"))
                    .map(|(mut vs, gs)| {
                        vs.grid_spacing = gs.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_3d_grid"))
                    .map(|(mut vs, s3g)| {
                        vs.show_3d_grid = s3g.parse().unwrap();
                        vs
                    })
                    .then(key_value("hide_objects"))
                    .map(|(mut vs, ho)| {
                        vs.hide_objects = ho.parse().unwrap();
                        vs
                    })
                    .then(key_value("hide_walls"))
                    .map(|(mut vs, hw)| {
                        vs.hide_walls = hw.parse().unwrap();
                        vs
                    })
                    .then(key_value("hide_stripes"))
                    .map(|(mut vs, hs)| {
                        vs.hide_stripes = hs.parse().unwrap();
                        vs
                    })
                    .then(key_value("hide_neighbors"))
                    .map(|(mut vs, hn)| {
                        vs.hide_neighbors = hn.parse().unwrap();
                        vs
                    })
                    .then(key_value("hide_detail"))
                    .map(|(mut vs, hd)| {
                        vs.hide_detail = hd.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_brushes"))
                    .map(|(mut vs, sb)| {
                        vs.show_brushes = sb.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_entities"))
                    .map(|(mut vs, se)| {
                        vs.show_entities = se.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_light_radius"))
                    .map(|(mut vs, slr)| {
                        vs.show_light_radius = slr.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_lighting_preview"))
                    .map(|(mut vs, slp)| {
                        vs.show_lighting_preview = slp.parse().unwrap();
                        vs
                    })
                    .then(key_value("show_wireframe"))
                    .map(|(mut vs, sw)| {
                        vs.show_wireframe = sw.parse().unwrap();
                        vs
                    }),
            )
            .then_ignore(close_block())
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::util::lex;

    use super::*;
    use chumsky::{input::Stream, Parser};
    use logos::Logos as _;
}
