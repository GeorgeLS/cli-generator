pub mod cpp;

pub(crate) fn left_pad<W: std::fmt::Write>(padding: usize, mut buffer: W) -> std::fmt::Result {
    write!(buffer, "{:padding$}", "")
}
