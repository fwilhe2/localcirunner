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

    let pipeline = parse_yaml_string(&contents);
    run_pipeline(pipeline);
}

fn run_pipeline(pipeline: Pipeline) {
    let commands = pipeline
        .steps
        .iter()
        .map(|step| {
            shell_command(
                step.shell_script.to_owned(),
                step.working_directory.to_owned(),
            )
        })
        .collect::<Vec<Command>>();

    for mut c in commands {
        let output = c.output().expect("Could not execute command");

        println!();

        // if name.as_str().is_some() {
        //     println!("Step: {}", name.as_str().unwrap())
        // }
        // println!("status: {}", output.status);
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());
    }
}

fn shell_command(shell_script: String, working_directory: String) -> Command {
    let mut my_command = Command::new("bash");
    my_command.arg("-c");

    // fixme is it ok not to split words here?
    my_command.arg(shell_script);

    println!("{:?}", my_command);

    return my_command;
}

fn parse_yaml_string(yaml: &str) -> Pipeline {
    let docs = YamlLoader::load_from_str(&yaml).unwrap();

    // Multi document support, first_yaml_document is of type yaml::Yaml
    let first_yaml_document = &docs[0];

    // Debug support
    // println!("{:?}", first_yaml_document);

    // azure
    let steps = &first_yaml_document["steps"];
    // println!("{:?}", steps);

    if first_yaml_document["steps"].is_badvalue() {
        println!("not azure");
        return github_yaml_to_pipeline(&first_yaml_document["jobs"]);
    }

    if first_yaml_document["jobs"].is_badvalue() {
        println!("not github");
        return azure_yaml_to_pipeline(&first_yaml_document["steps"]);
    }

    println!("xx {:?}", first_yaml_document["jobs"]);

    //github
    let gh_steps = &first_yaml_document["jobs"]["build"]["steps"];
    println!("{:?}", gh_steps);

    return Pipeline { steps: vec![] };
}

fn azure_yaml_to_pipeline(steps: &yaml_rust::Yaml) -> Pipeline {
    println!("xx {:?}", steps);

    let y = steps
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| Step {
            name: x["displayName"].as_str().unwrap_or("").to_string(),
            shell_script: x["script"].as_str().unwrap().to_string(),
            working_directory: x["workingDirectory"].as_str().unwrap_or("").to_string(),
        })
        .collect::<Vec<Step>>();

    return Pipeline { steps: y };
}

fn github_yaml_to_pipeline(jobs: &yaml_rust::Yaml) -> Pipeline {
    //fixme job name "build" is hardcoded
    let steps = &jobs["build"]["steps"];
    let y = steps
        .as_vec()
        .unwrap()
        .iter()
        .map(|x| Step {
            name: x["name"].as_str().unwrap_or("").to_string(),
            shell_script: x["run"].as_str().unwrap_or("echo nothing").to_string(),
            working_directory: x["working-directory"].as_str().unwrap_or("").to_string(),
        })
        .collect::<Vec<Step>>();

    return Pipeline { steps: y };
}

#[derive(Debug)]
pub struct Pipeline {
    steps: Vec<Step>,
}

impl PartialEq for Pipeline {
    fn eq(&self, other: &Self) -> bool {
        self.steps.len() == other.steps.len()
    }
}

#[derive(Debug)]
pub struct Step {
    shell_script: String,
    name: String,
    working_directory: String,
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_pipeline_1() {
        let input = "jobs:
    build:
      steps:
        - name: configure
          run: ./configure
        - name: make
          run: make";
        let actual = parse_yaml_string(&input);

        let expected = Pipeline {
            steps: vec![
                Step {
                    shell_script: "./configure".to_string(),
                    name: "configure".to_string(),
                    working_directory: "".to_string(),
                },
                Step {
                    shell_script: "make".to_string(),
                    name: "make".to_string(),
                    working_directory: "".to_string(),
                },
            ],
        };

        println!("{:#?}", actual);
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_pipeline_2() {
        let input = "steps:
  - script: ./configure
    workingDirectory: abc
  - script: make
    displayName: Compile the Code
  - script: make check";
        let actual = parse_yaml_string(&input);

        println!("{:#?}", actual);

        let expected = Pipeline {
            steps: vec![
                Step {
                    shell_script: "./configure".to_string(),
                    name: "".to_string(),
                    working_directory: "abc".to_string(),
                },
                Step {
                    shell_script: "make".to_string(),
                    name: "Compile the Code".to_string(),
                    working_directory: "".to_string(),
                },
                Step {
                    shell_script: "make check".to_string(),
                    name: "".to_string(),
                    working_directory: "".to_string(),
                },
            ],
        };

        assert_eq!(actual, expected)
    }
}
