use log::info;
use rbxlx_to_rojo::{filesystem::FileSystem, process_instructions};
use std::{
    fmt, fs,
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Debug)]
enum Problem {
    BinaryDecodeError(rbx_binary::DecodeError),
    InvalidFile,
    InvalidUsage,
    IoError(&'static str, io::Error),
    NFDCancel,
    NFDError(String),
    XMLDecodeError(rbx_xml::DecodeError),
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Problem::BinaryDecodeError(error) => write!(
                formatter,
                "While attempting to decode the place file, at {} rbx_binary didn't know what to do",
                error,
            ),

            Problem::InvalidFile => {
                write!(
                    formatter,
                    "Unsupported file extension. Supported: .rbxl, .rbxm, .rbxlx, .rbxmx"
                )
            }

            Problem::InvalidUsage => write!(
                formatter,
                "Invalid arguments. Use --help to see usage instructions."
            ),

            Problem::IoError(doing_what, error) => {
                write!(formatter, "While attempting to {}, {}", doing_what, error)
            }

            Problem::NFDCancel => write!(formatter, "No file or folder was selected."),

            Problem::NFDError(error) => write!(
                formatter,
                "Something went wrong when opening the file picker: {}",
                error,
            ),

            Problem::XMLDecodeError(error) => write!(
                formatter,
                "While attempting to decode the place file, at {} rbx_xml didn't know what to do",
                error,
            ),
        }
    }
}

struct WrappedLogger {
    log: env_logger::Logger,
    log_file: Arc<RwLock<Option<fs::File>>>,
}

impl log::Log for WrappedLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.log.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.log.log(record);

            if let Some(ref mut log_file) = &mut *self.log_file.write().unwrap() {
                log_file
                    .write(format!("{}\r\n", record.args()).as_bytes())
                    .ok();
            }
        }
    }

    fn flush(&self) {}
}

fn print_usage(program: &str) {
    println!("rbxl2rojo {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Supported input formats: .rbxl, .rbxm, .rbxlx, .rbxmx");
    println!();
    println!("Usage:");
    println!("  {} <place-file> <output-folder>", program);
    println!("  {}", program);
    println!();
    println!("Examples:");
    println!("  {}", program);
    println!("  {} game.rbxmx ./exports", program);
    println!();
    println!("Default: open file and folder dialogs.");
    println!("With two arguments: use paths directly from terminal.");
}

enum InputMode {
    Picker,
    Paths(PathBuf, PathBuf),
}

fn parse_cli_inputs() -> Result<Option<InputMode>, Problem> {
    let args = std::env::args().collect::<Vec<_>>();
    let program = args.first().map(String::as_str).unwrap_or("rbxl2rojo");

    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage(program);
        return Ok(None);
    }

    match args.len() {
        1 => Ok(Some(InputMode::Picker)),
        3 => Ok(Some(InputMode::Paths(
            PathBuf::from(&args[1]),
            PathBuf::from(&args[2]),
        ))),
        _ => {
            print_usage(program);
            Err(Problem::InvalidUsage)
        }
    }
}

fn read_inputs_from_picker() -> Result<(PathBuf, PathBuf), Problem> {
    let file_path = match nfd::open_file_dialog(Some("rbxl,rbxm,rbxlx,rbxmx"), None)
        .map_err(|error| Problem::NFDError(error.to_string()))?
    {
        nfd::Response::Okay(path) => PathBuf::from(path),
        nfd::Response::Cancel => return Err(Problem::NFDCancel),
        _ => return Err(Problem::InvalidUsage),
    };

    let default_root = file_path.parent().unwrap_or(Path::new("."));
    let root = match nfd::open_pick_folder(Some(&default_root.to_string_lossy()))
        .map_err(|error| Problem::NFDError(error.to_string()))?
    {
        nfd::Response::Okay(path) => PathBuf::from(path),
        nfd::Response::Cancel => return Err(Problem::NFDCancel),
        _ => return Err(Problem::InvalidUsage),
    };

    Ok((file_path, root))
}

fn routine() -> Result<(), Problem> {
    let input_mode = match parse_cli_inputs()? {
        Some(values) => values,
        None => return Ok(()),
    };

    let (file_path, mut root) = match input_mode {
        InputMode::Picker => read_inputs_from_picker()?,
        InputMode::Paths(file_path, root) => (file_path, root),
    };

    if !file_path.exists() {
        return Err(Problem::IoError(
            "read the place file",
            io::Error::new(io::ErrorKind::NotFound, "place file does not exist"),
        ));
    }

    if !root.exists() {
        fs::create_dir_all(&root).map_err(|error| Problem::IoError("create output folder", error))?;
    }

    if !root.is_dir() {
        return Err(Problem::IoError(
            "use output folder",
            io::Error::new(io::ErrorKind::InvalidInput, "output path is not a folder"),
        ));
    }

    let env_logger = env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .build();

    let log_file = Arc::new(RwLock::new(None));
    let logger = WrappedLogger {
        log: env_logger,
        log_file: Arc::clone(&log_file),
    };

    log::set_boxed_logger(Box::new(logger)).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("rbxlx-to-rojo {}", env!("CARGO_PKG_VERSION"));

    info!("Opening place file");
    let file_source = BufReader::new(
        fs::File::open(&file_path)
            .map_err(|error| Problem::IoError("read the place file", error))?,
    );
    info!("Decoding place file, this is the longest part...");

    let extension = file_path
        .extension()
        .map(|value| value.to_string_lossy().to_ascii_lowercase());

    let tree = match extension.as_deref() {
        Some("rbxmx") | Some("rbxlx") => {
            rbx_xml::from_reader_default(file_source).map_err(Problem::XMLDecodeError)
        }
        Some("rbxm") | Some("rbxl") => {
            rbx_binary::from_reader(file_source).map_err(Problem::BinaryDecodeError)
        }
        _ => Err(Problem::InvalidFile),
    }?;

    root = root.canonicalize().unwrap_or(root);

    let project_name = file_path.file_stem().ok_or(Problem::InvalidFile)?;
    let mut filesystem = FileSystem::from_root(root.join(project_name).into());

    log_file.write().unwrap().replace(
        fs::File::create(root.join("rbxl2rojo.log"))
            .map_err(|error| Problem::IoError("couldn't create log file", error))?,
    );

    info!("Starting processing, please wait a bit...");
    process_instructions(&tree, &mut filesystem);
    info!("Done! Check rbxl2rojo.log for a full log.");
    Ok(())
}

fn main() {
    if let Err(error) = routine() {
        eprintln!("An error occurred while using rbxlx-to-rojo.");
        eprintln!("{}", error);
    }
}
