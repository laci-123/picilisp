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

    pub fn step_line(self) -> Self {
        match self {
            Self::Native                      => self,
            Self::Prelude{line, column: _}    => Self::Prelude{    line: line + 1, column: 1 },
            Self::Stdin{line, column: _}      => Self::Stdin{      line: line + 1, column: 1 },
            Self::File{path, line, column: _} => Self::File{ path, line: line + 1, column: 1 },
        }
    }

    pub fn step_column(self) -> Self {
        match self {
            Self::Native                   => self,
            Self::Prelude{line, column}    => Self::Prelude{    line, column: column + 1 },
            Self::Stdin{line, column}      => Self::Stdin{      line, column: column + 1 },
            Self::File{path, line, column} => Self::File{ path, line, column: column + 1 },
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub read_name: String,
    pub location: Location,
    pub documentation: String,
    pub parameters: Vec<String>,
}
