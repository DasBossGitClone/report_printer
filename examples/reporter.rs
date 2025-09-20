use ::std::usize;

use ::reporter::{
    AnsiStyle, ChildLabel, Error, Label, RangeInclusive, ReportBuilder, Trim, TrimPadding,
};

fn single_line() {
    let mut report = ReportBuilder::new("Longer Test - Another test input");
    let label = Label::new(22..=26, "Overlapping label")
        .with_child_label(ChildLabel::new("Child label X"))
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn multi_line_label_and_childlabel() {
    let mut report = ReportBuilder::new("This is a test input string").with_range();
    let label = Label::new(5..=14, "This is a test label\nwith more than one line")
        .with_child_label(ChildLabel::new("This is a child label\nwith two lines"))
        .with_child_label(ChildLabel::new("This is another child label"));
    report.push(label);
    let label = Label::new(2..=4, "This is another label")
        .with_child_label(ChildLabel::new("Child label 1"))
        .with_child_label(ChildLabel::new("Child label 2"))
        .with_child_label(ChildLabel::new("Child label 3"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn with_color() {
    let mut report = ReportBuilder::new("Another test input");
    let label = Label::new(0..13, "A label at the start")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn multiline_overlapping_labels() {
    let mut report = ReportBuilder::new("Another test input, more text, even more text");
    let label = Label::new(3..=14, "A label at the start\nwith two lines")
        .with_child_label(ChildLabel::new("Child label A"))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let label = Label::new(7..=15, "Overlapping label")
        .with_child_label(ChildLabel::new("Child label X"))
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let label = Label::new(14..=26, "Another overlapping label")
        .with_child_label(ChildLabel::new("Child label 1"))
        .with_child_label(ChildLabel::new("Child label 2"))
        .with_child_label(ChildLabel::new("Child label 3\nwith two lines"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn write_iter() {
    let mut report = ReportBuilder::new("Longer Test - Another test input");
    let label = Label::new(22..=26, "Overlapping label")
        .with_child_label(ChildLabel::new("Child label X").with_color(AnsiStyle::ITALIC))
        .with_child_label(
            ChildLabel::new("Child label Y")
                //.with_color(AnsiStyle::UNDERLINE)
                .with_color(AnsiStyle::CYAN)
                .with_color(AnsiStyle::BOLD),
        );
    report.push(label);
    let label = Label::new(5..=14, "This is a test label\nwith more than one line")
        .with_child_label(ChildLabel::new("This is a child label\nwith two lines"))
        .with_child_label(ChildLabel::new("This is another child label"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    let writer = report.into_writer(&mut output);

    for res in writer {
        res.expect("Failed to write report");
    }

    print!("{}", String::from_utf8_lossy(&output));
}

fn no_labels() {
    let report = ReportBuilder::new("This is a test input string").with_range();
    assert_eq!(Error::NoLabels, report.finish().unwrap_err());
}

#[cfg(not(feature = "truncate_out_of_bounds"))]
fn out_of_bounds() {
    let mut report = ReportBuilder::new("Short input");
    let label = Label::new(0..20, "Out of bounds label");
    report.push(label);
    assert_eq!(
        Error::OutOfBounds {
            attempted_range: RangeInclusive { start: 0, end: 19 },
            valid: RangeInclusive { start: 0, end: 11 }
        },
        report.finish().unwrap_err()
    );
}
fn empty_input() {
    let mut report = ReportBuilder::new("");
    let label = Label::new(0..5, "Empty input label");
    report.push(label);
    assert_eq!(Error::EmptyInput, report.finish().unwrap_err());
}
fn empty_label_message() {
    let mut report = ReportBuilder::new("Non-empty input");
    let label = Label::new(0..5, "");
    report.push(label);
    assert_eq!(Error::LabelEmptyMessage, report.finish().unwrap_err());
}
fn empty_child_label_message() {
    let mut report = ReportBuilder::new("Non-empty input");
    let label = Label::new(0..5, "Non-empty label").with_child_label(ChildLabel::new(""));
    report.push(label);
    assert_eq!(Error::LabelChildEmptyMessage, report.finish().unwrap_err());
}

#[cfg(feature = "caret_color")]
fn single_label_with_color_and_caret_color() {
    let mut report = ReportBuilder::new("Another test input").caret_color(true);
    let label = Label::new(0..13, "A label at the start")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
#[cfg(feature = "caret_color")]
fn overlapping_labels_with_color_and_caret_color() {
    use ::token::RgbColor;

    let mut report =
        ReportBuilder::new("Another test input, more text, even more text").caret_color(true);
    let label = Label::new(3..=14, "A label at the start\nwith two lines")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let label = Label::new(7..=15, "Overlapping label")
        .with_color(AnsiStyle::YELLOW)
        .with_child_label(ChildLabel::new("Child label X").with_color(AnsiStyle::BLUE))
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let label = Label::new(14..=26, "Another overlapping label")
        .with_caret_color(RgbColor::BRIGHT_CYAN)
        .with_color(AnsiStyle::CYAN)
        .with_child_label(ChildLabel::new("Child label 1").with_color(AnsiStyle::MAGENTA))
        .with_child_label(ChildLabel::new("Child label 2").with_caret_color(RgbColor::GREEN))
        .with_child_label(
            ChildLabel::new("Child label 3\nwith two lines")
                .with_color(AnsiStyle::WHITE)
                .with_caret_color(RgbColor::RED),
        );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

#[cfg(not(feature = "merge_overlap"))]
fn directly_overlaping_labels() {
    let mut report = ReportBuilder::new("Test input for directly overlapping labels");
    let label = Label::new(5..=20, "First label")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let label = Label::new(5..=20, "Second label")
        .with_color(AnsiStyle::YELLOW)
        .with_child_label(ChildLabel::new("Child label X").with_color(AnsiStyle::BLUE))
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

#[cfg(feature = "merge_overlap")]
fn directly_overlaping_labels_merged() {
    let mut report = ReportBuilder::new("Test input for directly overlapping labels");
    let label = Label::new(5..=20, "First label")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let label = Label::new(5..=20, "Second label")
        .with_color(AnsiStyle::YELLOW)
        .with_child_label(ChildLabel::new("Child label X").with_color(AnsiStyle::BLUE))
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let label = Label::new(5..=20, "Third label")
        .with_color(AnsiStyle::CYAN)
        .with_child_label(ChildLabel::new("Child label 1").with_color(AnsiStyle::MAGENTA))
        .with_child_label(ChildLabel::new("Child label 2").with_color(AnsiStyle::WHITE));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
#[cfg(all(feature = "merge_overlap", feature = "caret_color"))]
fn directly_overlaping_labels_merged_with_caret_color() {
    use ::token::RgbColor;
    let mut report =
        ReportBuilder::new("Test input for directly overlapping labels").caret_color(true);
    let label = Label::new(5..=20, "First label")
        .with_caret_color(RgbColor::BRIGHT_GREEN)
        .with_color(AnsiStyle::GREEN)
        .with_child_label(
            ChildLabel::new("Child label A")
                .with_color(AnsiStyle::RED)
                .with_caret_color(RgbColor::BRIGHT_RED),
        )
        .with_child_label(ChildLabel::new("Child label B").with_caret_color(RgbColor::CYAN));
    report.push(label);
    let label = Label::new(5..=20, "Second label")
        .with_caret_color(RgbColor::BRIGHT_YELLOW)
        .with_color(AnsiStyle::YELLOW)
        .with_child_label(
            ChildLabel::new("Child label X")
                .with_color(AnsiStyle::BLUE)
                .with_caret_color(RgbColor::BRIGHT_BLUE),
        )
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let label = Label::new(5..=20, "Third label")
        .with_caret_color(RgbColor::BRIGHT_CYAN)
        .with_color(AnsiStyle::CYAN)
        .with_child_label(
            ChildLabel::new("Child label 1")
                .with_color(AnsiStyle::MAGENTA)
                .with_caret_color(RgbColor::MAGENTA),
        )
        .with_child_label(
            ChildLabel::new("Child label 2")
                .with_color(AnsiStyle::WHITE)
                .with_caret_color(RgbColor::WHITE),
        );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn multiline_overlapping_labels_colored_input() {
    let mut report = ReportBuilder::new("Another test input").colored_input(true);
    let label = Label::new(0..13, "A label at the start")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

#[cfg(feature = "truncate_out_of_bounds")]
fn out_of_bounds_truncate_silent() {
    use ::reporter::TruncateMode;

    let mut report = ReportBuilder::new("Short input").truncate_out_of_bounds(TruncateMode::Silent);
    let label = Label::new(0..20, "Out of bounds label");
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
#[cfg(feature = "truncate_out_of_bounds")]
fn out_of_bounds_truncate_indicated() {
    use ::reporter::TruncateMode;

    let mut report =
        ReportBuilder::new("Short input").truncate_out_of_bounds(TruncateMode::Indicate);
    let label = Label::new(0..20, "Out of bounds label");
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn very_long_label() {
    let mut report = ReportBuilder::new("Short input");
    let label = Label::new(
        0..5,
        "This is a very long label message that exceeds the input length significantly",
    );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn very_long_input_trimmed_long() {
    let mut report =
        ReportBuilder::new("Very long input that should be trimmed to show only relevant parts")
            .trim_input_padded((2, 2));
    let label = Label::new(
        10..=15,
        "This is a very long label message that exceeds the input length significantly",
    );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
fn very_long_input_trimmed_tight() {
    let mut report =
        ReportBuilder::new("Very long input that should be trimmed to show only relevant parts")
            .trim_input_padded((0, 0));
    let label = Label::new(
        10..=15,
        "This is a very long label message that exceeds the input length significantly",
    );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
fn very_long_input_trimmed_front() {
    let mut report =
        ReportBuilder::new("Very long input that should be trimmed to show only relevant parts")
            .trim_input_padded(1);
    let label = Label::new(
        10..=15,
        "This is a very long label message that exceeds the input length significantly",
    );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
fn very_long_input_trimmed_back() {
    let mut report =
        ReportBuilder::new("Very long input that should be trimmed to show only relevant parts")
            .trim_input_padded(TrimPadding::new(usize::MAX, 1));
    let label: Label = Label::new(
        21..=27,
        "This is a very long label message that exceeds the input length significantly",
    );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}
fn very_long_input_trimmed_chars() {
    let mut report =
        ReportBuilder::new("Very long input that should be trimmed to show only relevant parts")
            .trim_input(Trim::Chars(TrimPadding::new(3, 1)));
    let label = Label::new(
        21..=21,
        "This is a very long label message that exceeds the input length significantly",
    );
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    print!("{}", String::from_utf8_lossy(&output));
}

fn main() {
    very_long_input_trimmed_chars();
    println!("----------------------------------------");
    very_long_input_trimmed_back();
    println!("----------------------------------------");
    very_long_input_trimmed_front();
    println!("----------------------------------------");
    very_long_input_trimmed_long();
    println!("----------------------------------------");
    very_long_input_trimmed_tight();
    println!("----------------------------------------");
    very_long_label();
    println!("----------------------------------------");
    single_line();
    println!("----------------------------------------");
    multi_line_label_and_childlabel();
    println!("----------------------------------------");
    with_color();
    println!("----------------------------------------");
    multiline_overlapping_labels();
    println!("----------------------------------------");
    write_iter();
    println!("----------------------------------------");
    {
        // Functions with not outputs, we dont wanna print a separator for these
        no_labels();
        #[cfg(not(feature = "truncate_out_of_bounds"))]
        out_of_bounds();
        empty_input();
        empty_label_message();
        empty_child_label_message();
    }
    #[cfg(feature = "caret_color")]
    {
        println!("----------------------------------------");
        single_label_with_color_and_caret_color();
        println!("----------------------------------------");
        overlapping_labels_with_color_and_caret_color();
        println!("----------------------------------------");
    }
    #[cfg(feature = "merge_overlap")]
    {
        directly_overlaping_labels_merged();
    }
    #[cfg(not(feature = "merge_overlap"))]
    {
        directly_overlaping_labels();
    }
    #[cfg(all(feature = "merge_overlap", feature = "caret_color"))]
    {
        println!("----------------------------------------");
        directly_overlaping_labels_merged_with_caret_color();
    }
    println!("----------------------------------------");
    multiline_overlapping_labels_colored_input();
    #[cfg(feature = "truncate_out_of_bounds")]
    {
        println!("----------------------------------------");
        out_of_bounds_truncate_silent();
        println!("----------------------------------------");
        out_of_bounds_truncate_indicated();
    }
}
