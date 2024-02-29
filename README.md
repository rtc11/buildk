# BuildK
Simple build tool for Kotlin.

## 📜 Manifest
- Create an **empty** `buildk.toml` file in your project root.
- Cofigure your project if it differs from the defaults.

##### Manifest defaults
```toml
[project]
main = "Main.kt"
path = "<project>"
src = "<project>/src"
test = "<project>/test"
out = "<project>/out"

[dependencies]
org.jetbrains.kotlin.kotlin-stdlib = "1.9.22"

[test-dependencies]
org.junit.platform.junit-platform-console-standalone = "1.10.1"
org.junit.jupiter.junit-jupiter-api = "5.5.2"

[repositories]
mavenCentral = "https://repo1.maven.org/maven2"

[kotlin]
# Specify path to kotlin if not found on KOTLIN_HOME nor /usr/local/Cellar/kotlin/1.9.22/
```

Which gives the following project structure:

```
buildk.toml
src/
    Main.kt
test/
out/
  cache.json
  src/
    MainKt.class
```

See `buildk config` to see all the configuration.

# Dependencies
Dependencies are cached in $HOME/.buildk/cache/

# 🪄 Commands
Build
>  buildk build <br>
>  buildk build src <br>
>  buildk build test

Clean output
> buildk clean

Configuration
> buildk config

Dependency tree
> buildk deps <br>
> buildk deps 3 (depth 3)

Download missing dependencies
> buildk fetch <br>
> buildk fetch org.jetbrains.kotlin:kotlin-stdlib:1.9.22

Build fat jar
> buildk release

Run program
> buildk run <br>
> buildk run Filename

Test code
> buildk test

Show build tree
> buildk tree

| cmd | desc |
| --- | ---- |
| clean   | clean the project      |
| build   | build the project      |
| test    | test the project       |
| run     | run the project        |
| release | release the project    |
| fetch   | fetch the project      |
| tree    | list the build tree    |
| deps    | print the dependencies |
| help    | print this help        |

# 🚧 TODO
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
- [x] Tests are automatically found
- [x] Manually create build-tree based on project package/imports
- [ ] Resolve cyclic project package structure
- [ ] KSP (kotlin compiler plugin) must be implemented in kotlin. Used for generating smarter build-trees
- [x] Support repositories
- [ ] Add init command for setting up basic project
- [ ] Kotlinc must be first looked up in manifest before trying to look for env-vars and default locations
- [ ] When downloading deps and transitive deps fails, the imported dependency is cached and no transitive will be wodnloaded again
- [ ] Check out crossbeam
- [ ] Check out ripgrep
