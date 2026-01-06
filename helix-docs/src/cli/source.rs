use super::SourceCommands;
use crate::error::Result;

pub fn run(command: SourceCommands, _json: bool) -> Result<()> {
    match command {
        SourceCommands::List => {
            todo!("source list command")
        }
        SourceCommands::Remove { id, force: _ } => {
            todo!("source remove command: remove {id}")
        }
    }
}
