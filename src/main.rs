use std::env;
use std::fs;

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

    let steps = &doc["steps"];
    println!("{:?}", steps);
    println!("{:?}", steps[0]);

    for v in steps.as_vec().unwrap() {
        let x = &v["script"];

        println!("{}", x.as_str().unwrap());
    }

}