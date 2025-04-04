mod serverless_function;
mod utils;

use crate::serverless_function::{create_new_project, deploy_function};
use clap::{Arg, Command};
use std::process;

fn main() {
    let matches = Command::new("CLI")
        .version("1.0")
        .author("Akinlua Bolamigbe <bolamigbeakinlua@gmail.com>")
        .about("A simple CLI example")
        .subcommand(
            Command::new("create-function")
                .about("Creates a new function")
                .args([
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .required(true)
                        .help("The name of the function to create"),
                    Arg::new("runtime")
                        .short('r')
                        .default_value("go")
                        .long("runtime")
                        .value_name("RUNTIME")
                        .required(false)
                        .help("The runtime for the function"),
                ]),
        )
        .subcommand(
            Command::new("deploy-function")
                .about("Deploys an existing function")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .required(true)
                        .help("The name of the function to deploy"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("create-function", sub_matches)) => {
            if let Some(name) = sub_matches.get_one::<String>("name") {
                if let Some(runtime) = sub_matches.get_one::<String>("runtime") {
                    if let Err(err) = create_new_project(name, runtime) {
                        eprintln!("Error creating function: {}", err);
                        process::exit(1);
                    }
                } else {
                    eprintln!("Runtime parameter is required");
                    process::exit(1);
                }
            } else {
                eprintln!("Name parameter is required");
                process::exit(1);
            }
        }
        Some(("deploy-function", sub_matches)) => {
            if let Some(name) = sub_matches.get_one::<String>("name") {
                if let Err(err) = deploy_function(name) {
                    eprintln!("Error deploying function: {}", err);
                    process::exit(1);
                }
            } else {
                eprintln!("Name parameter is required");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Please use a valid subcommand. Run with --help for more information.");
            process::exit(1);
        }
    }
}
