#![feature(trivial_bounds)]
#![deny(unused)]
#![allow(dead_code)]

pub mod builder;
pub mod printer;
use builder::*;
use printer::*;
mod find_iter;
use ::token::AnsiStyle;
pub use find_iter::*;

fn main() {
    let mut report = ArgumentErrorReporter::new("This is a test input string").with_range();
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
    println!("{}", String::from_utf8_lossy(&output));

    let mut report = ArgumentErrorReporter::new("Longer Test - Another test input");
    let label = Label::new(22..=26, "Overlapping label")
        .with_child_label(ChildLabel::new("Child label X"))
        .with_child_label(ChildLabel::new("Child label Y"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    println!("{}", String::from_utf8_lossy(&output));

    let mut report = ArgumentErrorReporter::new("Another test input");
    let label = Label::new(0..13, "A label at the start")
        .with_color(AnsiStyle::GREEN)
        .with_child_label(ChildLabel::new("Child label A").with_color(AnsiStyle::RED))
        .with_child_label(ChildLabel::new("Child label B"));
    report.push(label);
    let report = report.finish().unwrap();
    let mut output = Vec::new();
    report.write(&mut output).unwrap();
    println!("{}", String::from_utf8_lossy(&output));

    let mut report = ArgumentErrorReporter::new("Another test input, more text, even more text");
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
    println!("{}", String::from_utf8_lossy(&output));
}
