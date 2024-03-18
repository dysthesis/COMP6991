mod errors;
mod parsers;

use clap::Parser;
use unsvg::Image;

use miette::{Context, GraphicalReportHandler, IntoDiagnostic, Result};

/// A simple program to parse four arguments using clap.
#[derive(Parser)]
struct Args {
    /// Path to a file
    file_path: std::path::PathBuf,

    /// Path to an svg or png image
    image_path: std::path::PathBuf,

    /// Height
    height: u32,

    /// Width
    width: u32,
}

fn main() {
    let args: Args = Args::parse();

    // Access the parsed arguments
    let file_path = args.file_path;
    let file = match std::fs::read_to_string(file_path)
        // Let miette handle the diagnostics for any file opening failure
        .into_diagnostic()
        .wrap_err("Failed to open file.")
    {
        Ok(res) => res,
        Err(e) => {
            println!("{e}");
            return;
        }
    };

    let image_path = args.image_path;
    let height = args.height;
    let width = args.width;

    // There should be no remainder string from the parser. This should be ensured by nom's all_consuming
    let parse: Result<Vec<parsers::Token>, errors::ParseError> =
        parsers::parse(parsers::Span::new(file.as_str()));
    let program = match parse {
        Ok(res) => res,
        Err(e) => {
            let mut s = String::new();
            GraphicalReportHandler::new()
                .render_report(&mut s, &e)
                .unwrap();
            println!("{s}");
            return;
        }
    };

    println!("{:?}", file.as_str());
    println!("{:?}", program);

    // let image = Image::new(width, height);

    // match image_path.extension().map(|s| s.to_str()).flatten() {
    //     Some("svg") => {
    //         let res = image.save_svg(&image_path);
    //         if let Err(e) = res {
    //             eprintln!("Error saving svg: {e}");
    //             return Err(());
    //         }
    //     }
    //     Some("png") => {
    //         let res = image.save_png(&image_path);
    //         if let Err(e) = res {
    //             eprintln!("Error saving png: {e}");
    //             return Err(());
    //         }
    //     }
    //     _ => {
    //         eprintln!("File extension not supported");
    //         return Err(());
    //     }
    // }

    // Ok(())
}
