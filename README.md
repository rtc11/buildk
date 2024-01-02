# BuildK
Small build tool for Kotlin.

# 📜 Howto
Create `buildk.toml` in your current work directory (cwd).

#### Defaults:
buildk home dir: $HOME/.buildk

##### Manifest
```toml
[project]
main = "Main.kt"
path = "<cwd>"
src = "<cwd>/src"
test = "<cwd>/test"
out = "<cwd>/out"
```
... which gives the following project structure:

```
buildk.toml
src/
    Main.kt
test/
out/
```

###### Dependencies
By default, maven repository is used to search for dependencies.

Dependencies are cached in $HOME/.buildk/cache/

The following format is used in toml: 

> <groupid>.<artifactid> = "<version>"

Add JUnit to your manifest for running tests:
```toml
[test-dependencies]
org.junit.jupiter.junit-jupiter-api = "5.10.1"
```

Add a dependency:
```toml
[dependencies]
com.google.code.gson.gson = "2.10.1"
```

# 🪄 Commands

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
- [ ] KSP (kotlin compiler plugin) must be implemented in kotlin. Used for generating smarter build-trees.
