use std::path::PathBuf;



#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Location {
    Native,
    Prelude{line: usize, column: usize},
    Stdin{line: usize, column: usize},
    File{path: PathBuf, line: usize, column: usize},
}

impl Location {
    pub fn get_file(&self) -> Option<PathBuf> {
        if let Self::File{path, line: _, column: _} = self {
            Some(path.clone())
        }
        else {
            None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Native => format!("Rust source"),
            Self::Prelude{line, column} => format!("Prelude:{line}:{column}"),
            Self::Stdin{line, column} => format!("stdin:{line}:{column}"),
            Self::File{path, line, column} => format!("{}:{line}:{column}", path.to_str().unwrap_or("<ERROR reading filepath>")),
        }
    }

    pub fn get_line(&self) -> Option<usize> {
        match self {
            Self::Native                         => None,
            Self::Prelude{line, column: _}       => Some(*line),
            Self::Stdin{line, column: _}         => Some(*line),
            Self::File{path: _, line, column: _} => Some(*line),
        }
    }

    pub fn get_column(&self) -> Option<usize> {
        match self {
            Self::Native                         => None,
            Self::Prelude{line: _, column}       => Some(*column),
            Self::Stdin{line: _, column}         => Some(*column),
            Self::File{path: _, line: _, column} => Some(*column),
        }
    }

    pub fn step_line(&mut self) {
        match *self {
            Self::Native                                    => {},
            Self::Prelude{ref mut line, ref mut column}     => { *line += 1; *column = 0},
            Self::Stdin  {ref mut line, ref mut column}     => { *line += 1; *column = 0},
            Self::File   {ref mut line, ref mut column, ..} => { *line += 1; *column = 0},
        }
    }

    pub fn step_column(&mut self) {
        match self {
            Self::Native                      => {},
            Self::Prelude{ref mut column, ..} => { *column += 1 },
            Self::Stdin  {ref mut column, ..} => { *column += 1 },
            Self::File   {ref mut column, ..} => { *column += 1 },
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub read_name: String,
    pub location: Location,
    pub documentation: String,
}
