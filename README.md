# Local CI Runner

Tool for running commands in ci workflow files.
The commands are executed on the local machine.
It is assumed that the required development tools and libraries (compiler, build tools etc) are installed.

This is **very** experimental code.

Local CI Runner is inspired by [nektos/act](https://github.com/nektos/act), but has another focus.
Local CI Runner does not try to run any `setup-xy` actions/tasks for installing development tooling in the right version, as it is assumed that the right tools are available on your development machine.
The goal is to get a shortcut to "running the CI pipeline locally".

## Example Usage

```
$ localcirunner .github/workflows/main.yml 

[STEP] 
"bash" "-c" "echo nothing"
nothing

[STEP] Setup java
"bash" "-c" "echo nothing"
nothing

[STEP] 
"bash" "-c" "mvn -v"
Apache Maven 3.6.3 (cecedd343002696d0abb50b32b541b8a6ba2883f)
Maven home: /home/florianw/software/apache-maven-3.6.3

[STEP] 
"bash" "-c" "mvn --batch-mode --no-transfer-progress verify"
[INFO] Scanning for projects...
...
```
