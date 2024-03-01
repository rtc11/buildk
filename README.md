<p><h1 align="center">BuildK</h1></p>

<p align="center">
  <img src="logo.png" alt="Build K logo" title="BuildK Logo" /> 
  <br>
  Simple build tool for Kotlin.
</p>

## 🛠️ Install
```shell
cargo install --git https://github.com/rtc11/buildk
```

## 📜 Manifest
An empty `buildk.toml` is required in project root.

See default configurations:
```shell
buildk config
```

Override default with:
```toml
[project]
main = "Main.kt"    # file with main()
path = "<cwd>"      # current working dir
src = "<cwd>/src"   # sources
test = "<cwd>/test" # test sources
out = "<cwd>/out"   # output dir

[dependencies]      # compile and runtime dependencies
org.jetbrains.kotlin.kotlin-stdlib = "1.9.22"

[test-dependencies] # test dependencies
org.junit.platform.junit-platform-console-standalone = "1.10.1"
org.junit.jupiter.junit-jupiter-api = "5.5.2"

[repositories]      # repositories for artifacts
mavenCentral = "https://repo1.maven.org/maven2"

[kotlin]            # kotlin location
# Specify path to kotlin if not found on KOTLIN_HOME nor /usr/local/Cellar/kotlin/1.9.22/
```

Which gives the following project structure:

```yaml
project
└── .buildk.toml                  # Manifest
    ├── src                       
    │   └── Main.kt               # Source code
    ├── test                      
    │   └── MainTest.kt           # Test code (JUnit 5)
    └── out
        ├── cache.json            # Build cache
        ├── app.jar               # Release (fat-jar)
        ├── src         
        │   └── Mainkt.class      # Compiled sources
        └── test
            └── MainTestkt.class  # Compiled test sources
```

## 🪄 Commands

Show commands
```shell
buildk help
```

```
A Kotlin build tool for the 21st century

Usage: buildk <COMMAND>

Commands:
  build, -b  Build the project
  clean, -c  Clean the output directory
  config     Show the project configuration
  deps       Print the dependencies
  fetch      Fetch the dependencies
  release    Create a release (jar)
  run, -r    Run the project
  test, -t   Run JUnit tests
  tree       Print the build tree
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## 🚧 TODO
- [ ] Java runtime dependency resolution
- [ ] Java compile time dependency resolution
- [ ] AVL trees for dependency graph?
- [ ] Resolve conflicting dependency versions
- [x] Resolve cyclic transitive dependencies
- [ ] Cache classpath (complete dependency graph per used dependency)
- [ ] Support multi module
- [ ] Create a project graph for enabling parallel compilation
- [ ] Early cut off improvements
  * Checking timestamp on files is not always sufficient. E.g. when adding a comment.
- [ ] Shared cache 
  * Build once on one machine, share the result in the cloud
- [ ] Platform libs must be configurable (e.g. junit or kotlin-std:1.9.22)
  - [x] Kotlin std
  - [ ] Test libs
- [x] Tests are automatically found
- [x] Manually create build-tree based on project package/imports
- [ ] KSP (kotlin compiler plugin) must be implemented in kotlin. Used for generating smarter build-trees
- [x] Configurable repositories
- [ ] Add init command for setting up basic project
- [x] Kotlinc must be first looked up in manifest before trying to look for env-vars and default locations
- [x] When downloading deps and transitive deps fails, the imported dependency is cached and no transitive will be wodnloaded again
- [ ] Use crossbeam for concurrency downloads?
- [ ] Use Rayon for parallel compilation?
- [ ] Use ripgrep for lexing/ksp/avl/build-tree?
- [ ] Use prodash instead of stdout?
