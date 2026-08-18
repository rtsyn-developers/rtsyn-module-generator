mod fs;
mod names;
mod render;
mod spec;

use std::env;
use std::path::PathBuf;

use fs::{copy_rendered_templates, PrepareOutput};
use render::RenderContext;
use spec::ModuleSpec;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1).collect())?;
    let spec = ModuleSpec::read_from(&args.spec_path)?;
    let context = RenderContext::new(spec)?;
    copy_rendered_templates(&context, &args.output_dir, args.prepare_output)
}

struct Args {
    prepare_output: PrepareOutput,
    spec_path: PathBuf,
    output_dir: PathBuf,
}

impl Args {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut force = false;
        let mut positional = Vec::new();

        for arg in args {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--force" => force = true,
                _ if arg.starts_with('-') => return Err(format!("unknown option `{arg}`")),
                _ => positional.push(arg),
            }
        }

        if positional.len() != 2 {
            return Err("usage: rtsyn-module-generator [--force] <spec.toml> <output-dir>".to_string());
        }

        Ok(Self {
            prepare_output: if force {
                PrepareOutput::Replace
            } else {
                PrepareOutput::Create
            },
            spec_path: PathBuf::from(&positional[0]),
            output_dir: PathBuf::from(&positional[1]),
        })
    }
}

fn print_help() {
    println!("usage: rtsyn-module-generator [--force] <spec.toml> <output-dir>");
    println!();
    println!("Generate an RTSyn C module from a module specification.");
}
