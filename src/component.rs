// this module is useful to deal with a single component
use std::path::Path;
use serde::Deserialize;
use std::fs::File;
use std::error::Error;
use std::fmt;
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
#[allow(non_snake_case)]
#[serde(default)]
pub struct Component {
    Name: String,
    VersionScript: String,
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
    Run: String,
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
    build,
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

pub fn load_from(cmpt: &Path) -> Result<Component,Box<dyn Error>> {
    if cmpt.exists() && cmpt.is_file() {
        let file = File::open(cmpt)?;
        let cmpt_obj: Component = serde_yaml::from_reader(file).unwrap();
        Ok(cmpt_obj)
    } else {
        Err(Box::new(LoadError::NoCmptFile{name: String::from(cmpt.to_string_lossy())}))
    }
}


#[test]
fn load_from_path() {
    let source = Path::new("./resources/virgilio_cpt.yaml");
    let dest = load_from(source);
    if let Ok(i) = dest {
        assert_eq!("Geogos", i.Name);
        assert_eq!("git describe --tags 2> /dev/null", i.VersionScript);
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
        assert!(false, format!("{:?}",dest.unwrap_err()))
    }
}