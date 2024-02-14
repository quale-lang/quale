//! Utils module contains help documentation.

pub(crate) fn usage() {
    print!(
        "usage: qcc [options] <quale-file>
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
    {:10}\t{:<20}
",
        "--help",
        "show this page",
        "--analyze",
        "run static analyzer",
        "-O0",
        "disable optimizations",
        "-O1",
        "enable first-level optimizations",
        "-Og",
        "enable all optimizations",
        "-o",
        "compiled output"
    );
}
