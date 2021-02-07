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
    let doc = &docs[0];

    // Debug support
    println!("{:?}", doc);

    // azure
    let steps = &doc["steps"];

    if steps.as_vec().is_some() {
        println!("{:?}", steps);

        for v in steps.as_vec().unwrap() {
            let x = &v["script"];

            println!("{}", x.as_str().unwrap());
        }
    }

    //github
    let x = &doc["jobs"]["build"]["steps"];
    println!("{:?}", x);

    if x.as_vec().is_some() {
        for v in x.as_vec().unwrap() {
            let xx = &v["run"];

            if xx.as_str().is_some() {
                println!("{}", xx.as_str().unwrap());
                let mut cmds: Vec<&str> = xx.as_str().unwrap().split(" ").collect();
                let mut mycmd = Command::new(cmds[0]);
                let u: Vec<_> = cmds.drain(1..).collect();
                                for cmd in u {
                    println!("{}", cmd);
                    mycmd.arg(cmd);
                }
                println!("{:?}", mycmd.output().expect("foo"));
                // Command::new("ls")
            }

        }
    }



}