use super::*;

use crate::helpers::palette::THEME_PRESETS;

/// How light a colour reads, for the checks about which end of the ramp a row
/// sits at. A named terminal colour has no channels to weigh.
fn lightness(colour: Color) -> f32 {
    let Color::Rgb(red, green, blue) = colour else {
        panic!("a mixed ramp is made of rgb: {colour:?}");
    };
    0.299 * f32::from(red) + 0.587 * f32::from(green) + 0.114 * f32::from(blue)
}

#[test]
fn every_row_of_a_wordmark_is_the_same_width() {
    for row in WIDE {
        assert_eq!(row.chars().count(), WIDE_WIDTH, "ragged row: {row:?}");
    }
    for row in NARROW {
        assert_eq!(row.chars().count(), NARROW_WIDTH, "ragged row: {row:?}");
    }
}

#[test]
fn every_wordmark_fits_the_width_it_is_drawn_at() {
    assert!(WIDE.iter().all(|row| row.chars().count() < WIDE_COLUMNS), "the wide wordmark has to leave room around it at the width that chooses it");
    assert!(NARROW.iter().all(|row| row.chars().count() < NARROW_COLUMNS));
    assert!(COMPACT.chars().count() < NARROW_WIDTH, "the fallback has to fit where the small wordmark does not");
}

#[test]
fn the_settings_header_never_takes_the_wide_wordmark() {
    assert_eq!(rows_for(WIDE_COLUMNS).len(), NARROW.len(), "fourteen rows before the first setting is too many");
    assert_eq!(rows_for(NARROW_WIDTH).len(), NARROW.len());
}

#[test]
fn the_splash_takes_the_biggest_that_fits() {
    assert_eq!(splash_rows_for(200).len(), WIDE.len());
    assert_eq!(splash_rows_for(WIDE_COLUMNS).len(), WIDE.len());
    assert_eq!(splash_rows_for(WIDE_COLUMNS - 1).len(), NARROW.len());
    assert_eq!(splash_rows_for(NARROW_COLUMNS).len(), NARROW.len());
    assert_eq!(splash_rows_for(NARROW_COLUMNS - 1), [COMPACT]);
}

#[test]
fn a_column_too_narrow_gets_the_word_instead() {
    assert_eq!(rows_for(NARROW_WIDTH - 1), [COMPACT]);
}

#[test]
fn both_tones_reach_something_on_every_size() {
    for total in [WIDE.len(), NARROW.len()] {
        let bright = bright_rows(total);
        assert!(bright > 0, "{total} rows left the lighter tone with nothing");
        assert!(bright < total, "{total} rows left the darker tone with nothing");
    }
}

#[test]
fn the_lighter_tone_is_the_top_third_rounded_up() {
    assert_eq!(bright_rows(10), 3);
    assert_eq!(bright_rows(5), 2, "a short wordmark still gets some");
    assert_eq!(bright_rows(1), 1);
    assert_eq!(bright_rows(0), 0);
}

#[test]
fn every_cell_the_wave_touches_rolls_through_the_ramp() {
    // The crest is cut into one band a tint, so a cell sees every one of them as the light goes by.
    let sweep = Sweep::over(&WIDE);
    let seen: std::collections::BTreeSet<usize> = (0..6000).filter_map(|ms| sweep.band(4, 30, Duration::from_millis(ms))).collect();

    assert_eq!(seen.into_iter().collect::<Vec<_>>(), (0..PROFILE.len()).collect::<Vec<_>>(), "a cell should stand in every band of the crest as it passes");
}

#[test]
fn at_rest_the_top_rows_are_the_lighter_purple_and_the_rest_are_darker() {
    let theme = Theme::classic();
    let ramp = tints(&theme);

    for rows in [WIDE.as_slice(), NARROW.as_slice()] {
        let top = tint(0, 0, rows, &theme, Duration::ZERO);
        let bottom = tint(rows.len() - 1, 0, rows, &theme, Duration::ZERO);

        assert_eq!(top, ramp[RESTING], "the top rows rest a step above the palette's purple");
        assert_eq!(bottom, ramp[RESTING + 1], "and the rest a step below it");
        assert!(lightness(top) > lightness(bottom), "the split has to read as lighter on top: {top:?} over {bottom:?}");
    }
}

#[test]
fn the_ramp_runs_from_the_lightest_purple_to_the_darkest() {
    // It moves in lightness alone, so every step of it has to be darker than the
    // one before -- a ramp that wandered would not read as one colour lit.
    for preset in THEME_PRESETS {
        let ramp = tints(&preset.theme);
        if !ramp.iter().all(|tint| matches!(tint, Color::Rgb(..))) {
            continue;
        }
        for pair in ramp.windows(2) {
            assert!(lightness(pair[0]) > lightness(pair[1]), "{} goes back up from {:?} to {:?}", preset.label, pair[0], pair[1]);
        }
    }
}

#[test]
fn the_word_ends_on_the_tail_the_drawn_ones_do() {
    assert!(COMPACT.starts_with("atriu"), "it is still the name: {COMPACT}");
    assert!(COMPACT.ends_with('ɱ'), "the hook is the tail, kept at one row: {COMPACT}");
}

#[test]
fn a_single_row_takes_the_lighter_tone() {
    let theme = Theme::classic();

    assert_eq!(tint(0, 0, &[COMPACT], &theme, Duration::ZERO), tints(&theme)[RESTING], "there is no second tone to fall into");
}

#[test]
fn a_single_row_is_all_bright() {
    assert_eq!(bright_rows(1), 1, "there is no second tone to split into");
    assert_eq!(bright_rows(0), 0);
}

/// What a row of a wordmark reads as at this moment, spans flattened back into
/// the characters they carry.
fn drawn(rows: &[&str], elapsed: Duration) -> Vec<String> {
    lines(rows, &Theme::classic(), elapsed).iter().map(|line| line.spans.iter().map(|span| span.content.as_ref()).collect()).collect()
}

/// A spread of instants across more than one sweep, for the checks that have to
/// hold whatever the wave is doing.
fn moments() -> impl Iterator<Item = Duration> {
    (0..240).map(|step| Duration::from_millis(step * 47))
}

#[test]
fn at_rest_every_wordmark_is_the_art_as_it_was_drawn() {
    for rows in [WIDE.as_slice(), NARROW.as_slice(), &[COMPACT]] {
        assert_eq!(drawn(rows, Duration::ZERO), rows.to_vec(), "nothing has moved yet, so nothing should have changed");
    }
}

#[test]
fn the_grain_groups_cover_the_art_and_do_not_overlap() {
    let alphabet: String = GRAINS.concat();

    for glyph in WIDE.concat().chars().chain(NARROW.concat().chars()).filter(|glyph| *glyph != ' ') {
        assert!(alphabet.contains(glyph), "{glyph:?} is drawn but belongs to no group, so it could never move");
    }
    for (index, group) in GRAINS.iter().enumerate() {
        for glyph in group.chars() {
            assert_eq!(GRAINS.iter().position(|other| other.contains(glyph)), Some(index), "{glyph:?} is in two groups");
        }
    }
}

#[test]
fn a_swapped_glyph_comes_from_its_own_group() {
    for elapsed in moments() {
        for (row, (was, now)) in WIDE.iter().zip(drawn(&WIDE, elapsed)).enumerate() {
            for (column, (before, after)) in was.chars().zip(now.chars()).enumerate() {
                if before == after {
                    continue;
                }
                let group = GRAINS.iter().find(|group| group.contains(before)).expect("a drawn glyph belongs to a group");
                assert!(group.contains(after), "{before:?} at {row},{column} became {after:?}, which is not its own weight");
            }
        }
    }
}

#[test]
fn a_blank_stays_blank() {
    for elapsed in moments() {
        for (was, now) in WIDE.iter().zip(drawn(&WIDE, elapsed)) {
            for after in was.chars().zip(now.chars()).filter(|(before, _)| *before == ' ').map(|(_, after)| after) {
                assert_eq!(after, ' ', "a hole in the letterform filled in at {elapsed:?}");
            }
        }
    }
}

#[test]
fn the_word_is_never_scrambled() {
    // Its letters are in none of the groups, so the fallback takes the colour
    // and nothing else -- a shuffled `atriuɱ` would spell something else.
    for elapsed in moments() {
        assert_eq!(drawn(&[COMPACT], elapsed), vec![COMPACT.to_owned()], "the word moved at {elapsed:?}");
    }
}

#[test]
fn every_row_keeps_its_width_however_long_it_has_been_up() {
    for elapsed in moments() {
        for row in drawn(&WIDE, elapsed) {
            assert_eq!(row.chars().count(), WIDE_WIDTH, "the swap has to be one glyph for one glyph: {row:?}");
        }
        assert!(drawn(&NARROW, elapsed).iter().all(|row| row.chars().count() == NARROW_WIDTH));
    }
}

#[test]
fn the_grain_keeps_moving_and_stays_sparse() {
    for elapsed in moments().skip(10) {
        let count: usize = WIDE.iter().zip(drawn(&WIDE, elapsed)).map(|(was, now)| was.chars().zip(now.chars()).filter(|(before, after)| before != after).count()).sum();
        assert!(count > 0, "the wordmark stood still at {elapsed:?}");
        assert!(count < 80, "{count} of 325 cells moved at {elapsed:?}, which is static, not shimmer");
    }
}

#[test]
fn the_wave_reaches_every_tint_over_one_sweep() {
    let theme = Theme::classic();
    let mut seen = Vec::new();

    for elapsed in moments() {
        for row in 0..WIDE.len() {
            for column in 0..WIDE_WIDTH {
                let colour = tint(row, column, &WIDE, &theme, elapsed);
                if !seen.contains(&colour) {
                    seen.push(colour);
                }
            }
        }
    }

    for colour in tints(&theme) {
        assert!(seen.contains(&colour), "the wave never reached {colour:?}, so a tint is being paid for and not used");
    }
}

#[test]
fn the_wave_is_quiet_at_both_ends_of_its_cycle() {
    // Nothing lit as the crest leaves and nothing lit as it comes back is what
    // keeps the wrap from showing as a jump.
    let sweep = Sweep::over(&WIDE);

    for step in [0, sweep.period - 1] {
        let elapsed = Duration::from_millis((step as u128 * SWEEP_MS / sweep.period as u128) as u64);
        for row in 0..WIDE.len() {
            assert!((0..WIDE_WIDTH).all(|column| sweep.band(row, column, elapsed).is_none()), "step {step} of {} still had light on row {row}", sweep.period);
        }
    }
}

#[test]
fn the_crest_leans_across_the_rows() {
    // A band straight down the columns would read as a wipe; the lean is what
    // makes it a sheen.
    let sweep = Sweep::over(&WIDE);
    let lit = |row: usize, elapsed| (0..WIDE_WIDTH).filter(|column| sweep.band(row, *column, elapsed).is_some()).min();
    let elapsed = Duration::from_millis(2000);

    let top = lit(4, elapsed).expect("the crest should be on the wordmark by now");
    let bottom = lit(10, elapsed).expect("and on every row of it at once");
    assert!(bottom + LEAN <= top, "the crest sat at {top} on row 4 and {bottom} on row 10, which is not a lean");
}

#[test]
fn every_theme_gets_a_ramp_whose_tints_are_all_different() {
    // A theme is free to map several of its roles onto one colour, and many do. The wave has to
    // have somewhere to travel whichever theme is on.
    for preset in THEME_PRESETS {
        let ramp = tints(&preset.theme);
        if !ramp.iter().all(|tint| matches!(tint, Color::Rgb(..))) {
            continue;
        }
        for (index, tint) in ramp.iter().enumerate() {
            assert!(!ramp[..index].contains(tint), "{} repeats {tint:?} at stop {index} of its ramp: {ramp:?}", preset.label);
        }
    }
}

#[test]
fn a_theme_with_nothing_to_mix_keeps_the_colours_it_has() {
    // Named terminal colours have no channels to interpolate, which is also what keeps the
    // monochrome theme monochrome rather than inventing six greys for it.
    let named = [Color::Red, Color::Red, Color::Magenta, Color::Magenta, Color::LightMagenta, Color::Blue];

    assert_eq!(distinct(named), named);
}

#[test]
fn the_sheen_meets_the_resting_colour_at_both_ends() {
    // What the profile is for: a sheen that stopped on its darkest step would meet the rest of the
    // wordmark on a hard line instead of easing back into it.
    assert_eq!(PROFILE[0], 0, "the sheen has to start where the wordmark rests");
    assert_eq!(PROFILE[PROFILE.len() - 1], 0, "and come back to it rather than ending dark");

    let darkest = PROFILE.iter().position(|step| *step == *PROFILE.iter().max().expect("a profile")).expect("a darkest step");
    assert!(darkest < PROFILE.len() - 1, "there has to be something after the dark section to ease back through");
    assert!(PROFILE[darkest..].windows(2).all(|pair| pair[0] > pair[1]), "and it has to lighten the whole way out: {PROFILE:?}");
}

#[test]
fn the_sheen_lightens_before_it_darkens() {
    // Read left to right the light runs to its lightest first and its darkest second, which is the
    // direction the ramp itself is written in.
    let lightest = PROFILE.iter().position(|step| *step == *PROFILE.iter().min().expect("a profile")).expect("a lightest step");
    let darkest = PROFILE.iter().position(|step| *step == *PROFILE.iter().max().expect("a profile")).expect("a darkest step");

    assert!(lightest < darkest, "the light section belongs before the dark one, got {lightest} and {darkest}");
}
