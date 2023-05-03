# BuildK
Kotlin package manager and build tool.

# Manifest
Create a `buildk.toml` in your project root.

```toml
[project]
main = "Main.kt" # default

[dependencies]
io.ktor.ktor-client-core = "2.3.0"

[test-dependencies]
org.junit.platform.junit-platform-console-standalone = "1.9.3"
```

# Commands
> buildk clean

> buildk build

> buildk test

> buildk run

> buildk release
