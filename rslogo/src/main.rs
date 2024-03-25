mod errors;
mod parsers;
mod tokens;
mod turtle;

use std::{path::PathBuf};

use clap::Parser;
// use unsvg::Image;

use miette::{Context, GraphicalReportHandler, IntoDiagnostic, Result};
use tokens::{Command, Program};

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

fn main() -> Result<()> {
    miette::set_panic_hook();
    let args: Args = Args::parse();

    // Access the parsed arguments
    let file_path = args.file_path;

    let file = match std::fs::read_to_string(file_path)
        // Let miette handle the diagnostics for any file opening failure
        .into_diagnostic()
        // Add some context to the error
        .wrap_err("Failed to open file.")
    {
        Ok(res) => res,
        Err(error) => return Err(error),
    };

    let _image_path: PathBuf = args.image_path;
    let _height: u32 = args.height;
    let _width: u32 = args.width;

    let commands: Vec<Command> =
        match crate::parsers::parse(Box::leak(file.to_string().into_boxed_str())) {
            Ok(res) => res,
            Err(e) => {
                println!("{:?}", e);
                let mut s = String::new();
                GraphicalReportHandler::new()
                    .render_report(&mut s, &e)
                    .unwrap();
                println!("{s}");
                return Err(e).into_diagnostic();
            }
        };

    let program: Program = Program::new(commands);

    println!("{:?}", program.commands);

    // There should be no remainder string from the parser. This should be ensured by nom's all_consuming

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

    Ok(())
}
