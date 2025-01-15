mod serverless_function;
mod utils;

use crate::serverless_function::{create_new_project, deploy_function};
use clap::{Arg, Command};

fn main() {
    let matches = Command::new("CLI")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("A simple CLI example")
        .subcommand(
            Command::new("create-service")
                .about("Creates a new service")
                .args([
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .required(true)
                        .help("The name of the service to create"),
                    Arg::new("runtime")
                        .short('r')
                        .default_value("go")
                        .long("runtime")
                        .value_name("RUNTIME")
                        .required(false)
                        .help("The name of the service to create"),
                ]),
        )
        .subcommand(
            Command::new("deploy-service")
                .about("Deploys an existing service")
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .required(false)
                        .help("The name of the service to deploy"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("create-service", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let runtime = sub_matches.get_one::<String>("runtime").unwrap();
            create_new_project(name, runtime);
        }
        Some(("deploy-service", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            deploy_function(name)
        }
        _ => {}
    }
}
