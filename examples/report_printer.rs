use ::report_printer::{AnsiStyle, ChildLabel, Error, Label, RangeInclusive, ReportBuilder};

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

fn main() {
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
    no_labels();
    out_of_bounds();
    empty_input();
    empty_label_message();
    empty_child_label_message();
}
