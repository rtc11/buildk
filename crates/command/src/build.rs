use std::path::{Path, PathBuf};

use cache::cache::Cache;
use manifest::config::Config;
use manifest::dependencies::DependenciesTools;
use process::kotlin::Kotlin;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::tree::Tree;
use crate::{Set, Command};

pub (crate) struct Build<'a> {
    config: &'a Config,
    kotlin: &'a Kotlin<'a>,
    tree: &'a Tree,
}

impl <'a> Command for Build<'a> {
    type Item = Set;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("build");
        match arg {
            Some(Set::Src) => output.apply(self.build_src()),
            Some(Set::Test) => output.apply(self.build_test()),
            _ => output.apply(self.build_src()).apply(self.build_test())
        }
    }
}

impl <'a> Build<'_> {
    pub fn new(config: &'a Config, kotlin: &'a Kotlin, tree: &'a Tree) -> Build<'a> {
        Build { config, kotlin, tree }
    }

    fn build_src(&mut self) -> BuildkOutput {
        let mut output = BuildkOutput::new("build src");
        let mut cache = Cache::load(&self.config.manifest.project.out.cache);
        let build_tree = self.tree.get_sorted_tree().expect("Failed to get sorted build tree");
        let changed_files: Vec<&PathBuf> = build_tree.iter().filter(|file| cache.not_cached(file)).collect();

        if changed_files.is_empty() {
            return output.conclude(PartialConclusion::CACHED).to_owned();
        }

        let cache_key = changed_files
            .iter()
            .map(|src| cache::file_fingerprint(src).expect("Faile to create extra fingerprint"))
            .reduce(|a, b| a + b)
            .unwrap_or(0);

        self.kotlin.builder()
            .workdir(&self.config.manifest.project.path)
            .target(&self.config.manifest.project.out.src)
            .sources(changed_files)
            .cache_key(cache_key)
            .compile(&mut output)
            .to_owned()
    }

    fn build_test(&mut self) -> BuildkOutput {
        let mut output = BuildkOutput::new("build test"); 

        if !self.config.manifest.project.test.is_dir(){
            return output.to_owned()
        }

        let project_test_libs = self.config.manifest.dependencies
            .test_deps()
            .iter()
            .filter(|dependency| dependency.is_cached())
            .map(|dependency| dependency.jar_absolute_path())
            .collect::<Vec<PathBuf>>();

        let test_libs = self.kotlin.test_libs();
        let mut classpath = vec![&self.config.manifest.project.out.src];
        classpath.extend(project_test_libs.iter());
        classpath.extend(test_libs.iter());

        self.kotlin.builder()
            .workdir(&self.config.manifest.project.path)
            .sources(vec![&self.config.manifest.project.test])
            .classpath(classpath)
            .target(&self.config.manifest.project.out.test)
            .compile(&mut output)
            .to_owned()
    }
}

trait IsCached {
    fn not_cached(&mut self, file: &Path) -> bool;
}

impl IsCached for Cache {
    fn not_cached(&mut self, _file: &Path) -> bool {
        //!matches!(self.cache_file(file), Ok(PartialConclusion::CACHED))
        true
    }
}
