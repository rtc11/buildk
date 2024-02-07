# JPMS - Java Platform Module System
Also called project Jigsaw.


## modules
All modules is dependent on java.base.

A module is typically a .jar file with a module-info.class file at root.
To use a module, include the jar into *modulepath* in stead of *classpath*.

### Create a module
> - app/
>    - module-info.java
>    - App.java
> - lib/
>    - module-info.java
>    - hello/
>        - Lib.java

Where `app/module-info.java` looks like this: 
```java
module app {
    requires lib;
} 
```

And `lib/module-info.java` looks like this:
```java
module lib {
    exports hello; 
}
```

To use a module compile-time but not runtime, you can use `requires static hello;`

