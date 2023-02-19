pub mod rule34xxx;
pub mod unified;

pub type PostIngestFunction = fn(std::time::Instant) -> Result<Vec<unified::Post>, &'static str>;