use std::env;
use std::fs;
use std::process::Command;

extern crate yaml_rust;
use yaml_rust::{YamlLoader};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    println!("file: {}", filename);

    let contents = fs::read_to_string(filename)
    .expect("Something went wrong reading the file");

    println!("With text:\n{}", contents);

    let docs = YamlLoader::load_from_str(&contents).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let first_yaml_document = &docs[0];

    // Debug support
    println!("{:?}", first_yaml_document);

    // azure
    let steps = &first_yaml_document["steps"];
    if steps.as_vec().is_some() {
        println!("{:?}", steps);

        for step in steps.as_vec().unwrap() {
            let shell_script = &step["script"];

            println!("{}", shell_script.as_str().unwrap());
        }
    }

    //github
    let gh_steps = &first_yaml_document["jobs"]["build"]["steps"];
    println!("{:?}", gh_steps);
    if gh_steps.as_vec().is_some() {
        for step in gh_steps.as_vec().unwrap() {
            let shell_script = &step["run"];
            let working_directory = &step["working-directory"];

            if shell_script.as_str().is_some() {
                println!("{}", shell_script.as_str().unwrap());
                let mut cmds: Vec<&str> = shell_script.as_str().unwrap().split(" ").collect();
                let mut mycmd = Command::new(cmds[0]);
                let u: Vec<_> = cmds.drain(1..).collect();
                                for cmd in u {
                    println!("{}", cmd);
                    mycmd.arg(cmd);
                }
                if working_directory.as_str().is_some() {
                    mycmd.current_dir(working_directory.as_str().unwrap());
                }
                println!("{:#?}", mycmd.output().expect("foo"));
            }
        }
    }
}