# BuildK
Small build tool for Kotlin.

# 📜 Howto
Create a `buildk.toml` in your project root.

```toml
[project]
main = "Main.kt"

[dependencies]

[test-dependencies]
org.junit.jupiter.junit-jupiter-api = "5.10.1"
```

#### Defaults:
buildk.home = $HOME/.buildk

project structure:
```
buildk.toml
src/
    Main.kt
test/
out/
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
