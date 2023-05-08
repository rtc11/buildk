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
org.junit.platform.junit-platform-console-standalone = "1.9.3"
```

# 🪄 Commands
* build
* clean
* fetch
* help
* list
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