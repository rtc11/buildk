# BuildK
Kotlin package manager and build tool.

# 📜 Manifest
Create a `buildk.toml` in your project root.

```toml
[project]
main = "Main.kt" # default

[dependencies]
io.ktor.ktor-client-core = "2.3.0"

[test-dependencies]
org.junit.jupiter.junit-jupiter-api= "5.10.1"
```

# 🪄 Commands
* build
* clean
* fetch
* help
* deps
* release
* run
* test

# 🚧 TODO
* Add libs to src code classpath
* Resolve conflicting dependency versions
* Cache classpath (complete dependency graph per used dependency)
* Support multi module
* Create a project graph for enabling parallel compilation
* IDE support
  * IntelliJ IDEA
* Early cut off
  * Checking timestamp on files is not always sufficient. E.g. adding a comment.
  * After compilation of a file, check if the result is identical to the previous result. Stop compiling dependent files if no changes detected.
* Shared cache 
  * Build once on one machine, share the result in the cloud
