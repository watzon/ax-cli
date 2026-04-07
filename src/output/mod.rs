pub mod json_fmt;
pub mod plain_fmt;
pub mod tree_fmt;

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    #[default]
    Plain,
    Tree,
    Json,
}
