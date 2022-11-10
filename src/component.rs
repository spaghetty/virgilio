// this module is useful to deal with a single component
use std::path::{Path, PathBuf};
use serde::Deserialize;
use std::fs::File;
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::collections::HashMap;


#[derive(Debug)]
pub enum LoadError {
    NoCmptFile{name: String}
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError::NoCmptFile{name: n} => write!(f, "component file '{}' can not be found", n),
        }
    }
}

impl Error for LoadError {
    fn description(&self) -> &str {
        "error loading the component file"
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}


#[derive(Deserialize, Debug, Default)]
#[allow(non_snake_case, dead_code)] //dead_code support to be removed soon
#[serde(default)]
pub struct Component {
    Name: String,
    #[serde(flatten)]
    Version: VersionType,
    SourceDir: String,
    Type: CmptType,
    SharedDirMountPoint: String,
    Ports: HashMap<u16, u16>,
    RemoteRepo: RemoteRepoType,
    CheckRunning: String,
    Envs: HashMap<String, String>,
    RunLinks: HashMap<String, String>,
    Images: HashMap<String, String>,
    Commands: HashMap<String, CommandType>,
    pub Run: String,
}

#[derive(Deserialize, Debug, PartialEq, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct VersionType {
    VersionNumber: String,
    VersionScript: String,
}


#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CmptType {
    normal,
    supervisored,
}

impl Default for CmptType {
    fn default() -> Self { 
        CmptType::normal 
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum RepoType {
    source,
    built,
    builded,
}

impl Default for RepoType {
    fn default() -> Self { 
        RepoType::source 
    }
}

#[derive(Deserialize, Debug, PartialEq, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct RemoteRepoType {
    Repo: String,
    Type: RepoType,
}


#[derive(Deserialize, Debug, PartialEq, Default)]
#[allow(non_snake_case)]
#[serde(default)]
pub struct CommandType {
    Type: TypeCommandType,
    Command: String,
    Image: String,
    LinkSet: String,
    HostMask: String,
    User: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum TypeCommandType {
    Base,
    External,
    Exec,
}
impl Default for TypeCommandType {
    fn default() -> Self { 
        TypeCommandType::Base 
    }
}
pub fn load_from_reader<R: Read>(cmpt: R) -> Result<Component, Box<dyn Error>> {
    let cmpt_obj: Component = serde_yaml::from_reader(cmpt).unwrap();
    Ok(cmpt_obj)
}

pub fn load_from_pathbuf(cmpt: &PathBuf) -> Result<Component,Box<dyn Error>> {
    load_from(Path::new(cmpt))
}
pub fn load_from(cmpt: &Path) -> Result<Component,Box<dyn Error>> {
    if cmpt.exists() && cmpt.is_file() {
        let file = File::open(cmpt)?;
        load_from_reader(file)
    } else {
        Err(Box::new(LoadError::NoCmptFile{name: String::from(cmpt.to_string_lossy())}))
    }
}


#[test]
fn load_from_path() {
    let source = Path::new("./resources/Component.yaml");
    let dest = load_from(source);
    if let Ok(i) = dest {
        assert_eq!("Geogos", i.Name);
        assert_eq!("git describe --tags 2> /dev/null", i.Version.VersionScript);
        assert_eq!("src/gitlab.subito.int/development/geogos", i.SourceDir);
        assert!(CmptType::supervisored == i.Type);
        let port_mapped = i.Ports.get(&9996);
        match port_mapped {
            Some(&x) => assert_eq!(9996, x),
            None => assert!(false, "missing key in Ports"),
        }
        let repotype = i.RemoteRepo;
        assert_eq!("git@gitlab.subito.int:development/geogos.git",repotype.Repo);
        assert_eq!(RepoType::source, repotype.Type);
        assert_eq!("echo \"check-status\";  curl -o /dev/null --silent --write-out '%{http_code}\\n' http://localhost:9996/v1/geo/regions", i.CheckRunning);
        let declared_env = i.Envs.get("SERVICE_PORT");
        match declared_env {
            Some(x) => assert_eq!("9996",x),
            None => assert!(false, "missing key in Envs"),
        }
        let declared_links = i.RunLinks.get("Core");
        match declared_links {
            Some(x) => assert_eq!("trans", x),
            None => assert!(false, "missing key for links"),
        }
        let declared_images = i.Images.get("Default");
        match declared_images {
            Some(x) => assert_eq!("mesos-registry.subito.dev:5000/regress_golang", x),
            None => assert!(false, "missing key 'Default' in Images"),
        }
        let declared_command = i.Commands.get("Command");
        match declared_command {
            Some(x) => assert_eq!("go build -ldflags \"-X main.ServiceVersion {{.VersionNumber}}\" geo.go", x.Command),
            None => assert!(false, "missing key for 'Command' command"),
        }
        let declared_command = i.Commands.get("Unit");
        match declared_command {
            Some(x) => assert_eq!(TypeCommandType::Base, x.Type),
            None => assert!(false, "missing key for 'Unit' command"),
        }
        assert_eq!("./geo -address=:9996 -syslog=false -trans=trans:20205",i.Run)

    } else {
        assert!(false, "{:?}",dest.unwrap_err())
    }
}

#[test]
fn load_from_string() {
    let raw_cmpt = r###"
    Name: MainDB                     #component name
    VersionNumber: "9.5"               #service version
    RemoteRepo:
        Repo: ''
        Type: builded                  #builded components means just to get an image
        Reference: "9.5"                 #version used for builded image
    CheckRunning: psql -U postgres -h localhost -l | grep "\<postgres\>" ; exit \$? #command that will check if the component is running
    Distribute:
        Run:
            Run: IMG_DEFAULT
            Image: 'postgres'
    Run: IMG_DEFAULT
    Commands:
        BashIn:
            Type: 'Exec'
            Command: bash
            Interactive: true
    "###;
    let dest = load_from_reader(raw_cmpt.as_bytes());
    if let Ok(cmpt) = dest {
        assert_eq!("MainDB", cmpt.Name);
        assert_eq!("9.5", cmpt.Version.VersionNumber);
        assert!(cmpt.Version.VersionScript.is_empty());
    } else {
        assert!(false, "{:?}", dest.unwrap_err());
    }
}
