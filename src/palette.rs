use clap::builder::styling::{Color, RgbColor, Style, Styles};
use colored::{ColoredString, Colorize};

// The grey the web app uses for its labels
const LABEL_GREY: (u8, u8, u8) = (0xa3, 0xa3, 0xa3);

/// A color palette, mirroring the `palette-*` classes of the web app so that
/// the terminal summary and the web page agree on the colors of a given year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    // The darkest color of the palette, used for the frame
    color_1: (u8, u8, u8),
    // The accent color, used for the title
    color_2: (u8, u8, u8),
    // The two colors the web app builds its highlight gradient from
    color_3: (u8, u8, u8),
    color_4: (u8, u8, u8),
}

// Keep these in sync with `web/css/styles.css`
const DEFAULT: Palette = Palette {
    color_1: (0x14, 0x3F, 0x6B),
    color_2: (0xF5, 0x53, 0x53),
    color_3: (0xFE, 0xB1, 0x39),
    color_4: (0xF6, 0xF5, 0x4D),
};

const PALETTE_2023: Palette = Palette {
    color_1: (0x22, 0x09, 0x2C),
    color_2: (0x87, 0x23, 0x41),
    color_3: (0xBE, 0x31, 0x44),
    color_4: (0xF0, 0x59, 0x41),
};

const PALETTE_2022: Palette = Palette {
    color_1: (0x0E, 0x13, 0x1F),
    color_2: (0x38, 0x40, 0x5F),
    color_3: (0x59, 0x54, 0x6C),
    color_4: (0xFF, 0x00, 0x35),
};

const PALETTE_2021: Palette = Palette {
    color_1: (0x3A, 0x50, 0x6B),
    color_2: (0x6C, 0x22, 0xA6),
    color_3: (0x5B, 0xC0, 0xBE),
    color_4: (0x6F, 0xFF, 0xE9),
};

impl Palette {
    /// The palette the web app would pick for `year`.
    pub fn for_year(year: u32) -> Self {
        match year {
            2021 => PALETTE_2021,
            2022 => PALETTE_2022,
            2023 => PALETTE_2023,
            _ => DEFAULT,
        }
    }

    // The web app paints `color_1` and `color_2` onto the background, where
    // they are the darkest colors of the palette. A terminal only ever paints
    // text, so the colors below stick to `color_3` and `color_4` and mix the
    // darker colors in rather than using them on their own: as a foreground
    // they are illegible on a dark background, and they collapse to black on
    // terminals without true color support, where `colored` falls back to the
    // nearest of eight colors.

    /// The color of a highlighted number. The web app draws these with a
    /// gradient, which a terminal cannot do, so use the brighter of the two.
    pub fn highlight(&self, text: &str) -> ColoredString {
        paint(text, self.color_4).bold()
    }

    /// The accent color, for the title and the names of the leaderboard.
    pub fn accent(&self, text: &str) -> ColoredString {
        paint(text, self.color_3)
    }

    /// The color of a secondary number, for the counts of the leaderboard.
    pub fn secondary(&self, text: &str) -> ColoredString {
        paint(text, mix(self.color_3, self.color_4))
    }

    /// The color of the frame around the title.
    pub fn frame(&self, text: &str) -> ColoredString {
        paint(text, mix(self.color_2, self.color_3))
    }
}

fn paint(text: &str, (r, g, b): (u8, u8, u8)) -> ColoredString {
    text.truecolor(r, g, b)
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8)) -> (u8, u8, u8) {
    let mix_channel = |a: u8, b: u8| (((a as u16) + (b as u16)) / 2) as u8;
    (
        mix_channel(a.0, b.0),
        mix_channel(a.1, b.1),
        mix_channel(a.2, b.2),
    )
}

/// Dimmed text, for labels and units.
pub fn label(text: &str) -> ColoredString {
    paint(text, LABEL_GREY)
}

/// The color of an error message, matching how clap styles its own errors.
pub fn error(text: &str) -> ColoredString {
    paint(text, DEFAULT.color_2).bold()
}

/// The styles clap uses for the help output and its own error messages.
///
/// The year, and with it the palette of the run, is only known once the config
/// file has been read, which happens after the arguments have been parsed, so
/// the help output always uses the default palette.
pub fn clap_styles() -> Styles {
    Styles::styled()
        .header(styled(DEFAULT.color_3).bold().underline())
        .usage(styled(DEFAULT.color_3).bold().underline())
        .literal(styled(DEFAULT.color_4).bold())
        .placeholder(styled(LABEL_GREY))
        .valid(styled(DEFAULT.color_4))
        .invalid(styled(DEFAULT.color_2).bold())
        .error(styled(DEFAULT.color_2).bold())
}

fn styled((r, g, b): (u8, u8, u8)) -> Style {
    Style::new().fg_color(Some(Color::Rgb(RgbColor(r, g, b))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_year_with_a_web_palette_has_a_terminal_palette() {
        assert_eq!(Palette::for_year(2021), PALETTE_2021);
        assert_eq!(Palette::for_year(2022), PALETTE_2022);
        assert_eq!(Palette::for_year(2023), PALETTE_2023);
    }

    #[test]
    fn years_without_a_web_palette_fall_back_to_the_default() {
        // The web app only defines palettes for 2021 to 2023
        assert_eq!(Palette::for_year(2020), DEFAULT);
        assert_eq!(Palette::for_year(2025), DEFAULT);
    }

    #[test]
    fn mixing_averages_every_channel() {
        assert_eq!(mix((0, 0, 0), (255, 255, 255)), (127, 127, 127));
        assert_eq!(mix((10, 20, 30), (30, 40, 50)), (20, 30, 40));
    }
}
