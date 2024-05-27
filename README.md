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

Properties with defaults:
```toml
[project]
path = "<cwd>"
src  = "<cwd>/src"
test = "<cwd>/test"
main = "Main.kt"
out  = "<cwd>/out"

[compile]
org.jetbrains.kotlin.kotlin-stdlib = "1.9.22"

[runtime]

[test]
org.junit.platform.junit-platform-console-standalone = "1.10.1"
org.junit.jupiter.junit-jupiter-api = "5.5.2"

[repos]
mavenCentral = "https://repo1.maven.org/maven2"

[kotlin]
path = "/usr/local/Cellar/kotlin/1.9.22/"

[java]
path = "/usr/local/Cellar/openjdk/17.0.1/"
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

Usage: buildk [OPTIONS] <COMMAND>

Commands:
  build, -b  Build the project
  clean, -c  Clean the output directory
  config     Show the project configuration
  deps       Print the dependencies
  fetch      Fetch the dependencies
  init       Initialize the project
  release    Create a release (jar)
  run, -r    Run the project
  test, -t   Run JUnit tests
  tree       Print the build tree
  path       
  help       Print this message or the help of the given subcommand(s)

Options:
  -q             
  -h, --help     Print help
  -V, --version  Print version
```

## Dev
Faster builds with rayon (currently only with nightly)
```shell
RUSTFLAGS="-Z threads=8" cargo +nightly build --release
```

## 🚧 TODO
- [ ] Java runtime dependency resolution
- [ ] Java compile time dependency resolution
- [ ] AVL trees for dependency graph?
- [x] Resolve maven artifacts
- [x] Resolve gradle dependencies
- [x] Resolve buildk packages
- [ ] Resolve local packages
- [ ] Resolve git packages
- [ ] Publish buildk packages
- [ ] Buildk published packages can be stored on github package registry?
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
  - [ ] Java std
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
