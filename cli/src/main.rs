mod serverless_function;
mod utils;

use crate::serverless_function::{create_new_project, deploy_function};
use clap::{Arg, Command};

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
                        .help("The name of the function to create"),
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
                        .required(false)
                        .help("The name of the function to deploy"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("create-function", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let runtime = sub_matches.get_one::<String>("runtime").unwrap();
            create_new_project(name, runtime);
        }
        Some(("deploy-function", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            deploy_function(name)
        }
        _ => {}
    }
}
