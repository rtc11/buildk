use std::{
    fs::{create_dir, File},
    io::Write,
};

use util::buildk_output::BuildkOutput;

use crate::Command;

pub(crate) struct Init;

impl<'a> Command for Init {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("init");

        let cwd = std::env::current_dir().expect("path to current working directory");

        let buildk = cwd.join("buildk.toml");
        File::create(buildk).expect("Unable to create buildk.tom");

        create_dir(cwd.join("src")).expect("Unable to create src directory");
        let main = cwd.join("src").join("Main.kt");
        let mut main = File::create(main).expect("Unable to create Main.kt");
        main.write_all(b"fun main() {\n    println(\"Hello, World!\")\n}")
            .expect("Unable to write to Main.kt");

        create_dir(cwd.join("test")).expect("Unable to create test directory");
        let test = cwd.join("test").join("MainTest.kt");
        let mut test = File::create(test).expect("Unable to create MainTest.kt");
        test.write_all(b"import org.junit.jupiter.api.Test\nimport kotlin.test.assertEquals\n\nclass MainTest {\n    @Test\n    fun test() {\n        assertEquals(1, 1)\n    }\n}")
            .expect("Unable to write to MainTest.kt");

        output.conclude(util::PartialConclusion::SUCCESS).to_owned()
    }
}

impl Init {
    pub fn new() -> Init {
        Init {}
    }
}
