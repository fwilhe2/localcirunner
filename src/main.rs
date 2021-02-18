use std::env;
use std::fs;
use std::process::{Command, Stdio};

extern crate yaml_rust;
use yaml_rust::YamlLoader;

// todo:
// - default environment variables
// - non-bash shells
// - make output more pretty
// - debug mode
// - allow to configure job for gh workflows

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

fn parse_yaml_string(yaml: &str) -> Pipeline {
    let docs = YamlLoader::load_from_str(&yaml).unwrap();

    // Multi document support, first_yaml_document is of type yaml::Yaml
    let first_yaml_document = &docs[0];

    if first_yaml_document["steps"].is_badvalue() {
        return github_yaml_to_pipeline(&first_yaml_document["jobs"]);
    }

    if first_yaml_document["jobs"].is_badvalue() {
        return azure_yaml_to_pipeline(&first_yaml_document["steps"]);
    }

    panic!("should not get here, debug info: {:?}", first_yaml_document)
}

fn run_pipeline(pipeline: Pipeline) {
    let commands = pipeline
        .steps
        .iter()
        .map(|step| {
            shell_command(
                step.shell_script.to_owned(),
                step.working_directory.to_owned(),
                step.name.to_owned(),
                step.shell_interpreter.to_owned(),
            )
        })
        .collect::<Vec<MyCommand>>();

    for mut c in commands {
        println!("\n[STEP] {}", c.name);

        let mut cmd = c
            .command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();

        let status = cmd.wait().expect("Could not execute command");
        assert!(status.success());
    }
}

struct MyCommand {
    command: Command,
    name: String,
}

fn shell_command(
    shell_script: String,
    working_directory: String,
    name: String,
    shell_interpreter: String,
) -> MyCommand {
    let mut my_command = Command::new(shell_interpreter_with_default(shell_interpreter));
    my_command.arg("-c");

    // fixme is it ok not to split words here?
    my_command.arg(shell_script);

    if !working_directory.is_empty() {
        my_command.current_dir(working_directory);
    }

    MyCommand {
        command: my_command,
        name,
    }
}

fn shell_interpreter_with_default(shell_interpreter: String) -> String {
    if shell_interpreter.is_empty() {
        if cfg!(windows) {
            return "pwsh".to_string();
        } else if cfg!(unix) {
            return "bash".to_string();
        }
    }

    shell_interpreter
}

fn azure_yaml_to_pipeline(steps: &yaml_rust::Yaml) -> Pipeline {
    return Pipeline {
        steps: steps
            .as_vec()
            .unwrap()
            .iter()
            .map(|step| Step {
                name: step["displayName"].as_str().unwrap_or("").to_string(),
                shell_script: step["script"]
                    .as_str()
                    .unwrap_or("echo nothing")
                    .to_string(),
                working_directory: step["workingDirectory"].as_str().unwrap_or("").to_string(),
                shell_interpreter: "".to_string(), //fixme this seems not so simple https://docs.microsoft.com/en-us/azure/devops/pipelines/yaml-schema?view=azure-devops&tabs=schema%2Cparameter-schema#script
            })
            .collect::<Vec<Step>>(),
    };
}

fn github_yaml_to_pipeline(jobs: &yaml_rust::Yaml) -> Pipeline {
    //fixme job name "build" is hardcoded
    let steps = &jobs["build"]["steps"];

    return Pipeline {
        steps: steps
            .as_vec()
            .unwrap()
            .iter()
            .map(|step| Step {
                name: step["name"].as_str().unwrap_or("").to_string(),
                shell_script: step["run"].as_str().unwrap_or("echo nothing").to_string(),
                working_directory: step["working-directory"].as_str().unwrap_or("").to_string(),
                shell_interpreter: step["shell"].as_str().unwrap_or("").to_string(),
            })
            .collect::<Vec<Step>>(),
    };
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
    shell_interpreter: String,
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_pipeline_github() {
        let input =
            fs::read_to_string("test_data/gh.yaml").expect("Something went wrong reading the file");

        let actual = parse_yaml_string(&input);

        let expected = Pipeline {
            steps: vec![
                Step {
                    shell_script: "echo nothing".to_string(),
                    name: "".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "./configure".to_string(),
                    name: "configure".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "make".to_string(),
                    name: "make".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "ls\npwd\n".to_string(),
                    name: "Test Multiline String".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "pwd; echo *".to_string(),
                    name: "test in dir".to_string(),
                    working_directory: "./dir".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "pwd".to_string(),
                    name: "".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "echo ${env:PATH}".to_string(),
                    name: "Display the path".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "powershell".to_string(),
                },
                Step {
                    shell_script: "import os\nprint(os.environ[\'PATH\'])\n".to_string(),
                    name: "Display the path".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "python".to_string(),
                },
            ],
        };

        assert_eq!(actual, expected)
    }

    #[test]
    fn test_pipeline_azure() {
        let input = fs::read_to_string("test_data/azure.yaml")
            .expect("Something went wrong reading the file");

        let actual = parse_yaml_string(&input);

        let expected = Pipeline {
            steps: vec![
                Step {
                    shell_script: "./configure".to_string(),
                    name: "".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "make".to_string(),
                    name: "Compile the Code".to_string(),
                    working_directory: "".to_string(),
                    shell_interpreter: "".to_string(),
                },
                Step {
                    shell_script: "pwd; echo *".to_string(),
                    name: "test in dir".to_string(),
                    working_directory: "./dir".to_string(),
                    shell_interpreter: "".to_string(),
                },
            ],
        };

        assert_eq!(actual, expected)
    }
}
