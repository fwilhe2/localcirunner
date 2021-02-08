use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process::Command;

extern crate yaml_rust;
use yaml_rust::YamlLoader;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Missing required argument filename.\n  Usage: localcirunner .github/workflows/test.yml")
    }
    let filename = &args[1];
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");
    let docs = YamlLoader::load_from_str(&contents).unwrap();

    // Multi document support, first_yaml_document is of type yaml::Yaml
    let first_yaml_document = &docs[0];

    // Debug support
    // println!("{:?}", first_yaml_document);

    // azure
    let steps = &first_yaml_document["steps"];
    if steps.as_vec().is_some() {
        // println!("{:?}", steps);

        for step in steps.as_vec().unwrap() {
            let shell_script = &step["script"];
            let working_directory = &step["workingDirectory"];
            let name = &step["displayName"];

            if shell_script.as_str().is_some() {
                run_script(shell_script, name, working_directory)
            }
        }
    }

    //github
    let gh_steps = &first_yaml_document["jobs"]["build"]["steps"];
    // println!("{:?}", gh_steps);
    if gh_steps.as_vec().is_some() {
        for step in gh_steps.as_vec().unwrap() {
            let shell_script = &step["run"];
            let working_directory = &step["working-directory"];
            let name = &step["name"];

            if shell_script.as_str().is_some() {
                run_script(shell_script, name, working_directory)
            }
        }
    }
}

fn run_script(
    shell_script: &yaml_rust::Yaml,
    name: &yaml_rust::Yaml,
    working_directory: &yaml_rust::Yaml,
) {
    // println!("{}", shell_script.as_str().unwrap());
    let shell_script_lines: Vec<&str> = shell_script.as_str().unwrap().split("\n").collect();
    for line in shell_script_lines {
        let mut shell_script_words: Vec<&str> = line.trim().split(" ").collect();
        let mut my_command = Command::new(shell_script_words[0]);
        let command_arguments: Vec<_> = shell_script_words.drain(1..).collect();
        for argument in command_arguments {
            my_command.arg(argument);
        }
        if working_directory.as_str().is_some() {
            my_command.current_dir(working_directory.as_str().unwrap());
        }
        // println!(
        //     "{:#?}",
        //     my_command.output().expect("Could not execute command")
        // );

        let output = my_command.output().expect("Could not execute command");

        println!();

        if name.as_str().is_some() {
            println!("Step: {}", name.as_str().unwrap())
        }
        // println!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());
    }
}
