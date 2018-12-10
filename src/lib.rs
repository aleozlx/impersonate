extern crate regex;
extern crate nix;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use regex::Regex;

extern "C" {
    fn _su(user: *mut std::os::raw::c_char) -> i32;
}

const ROOT_UID: nix::libc::uid_t = 0;

pub struct User {
    user_name: String,
    group_name: String,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid
}

impl User {
    pub fn etc_passwd(&self) -> String {
        format!("{username}:x:{uid}:{gid}:{username},,,:/home/{username}:/bin/bash",
            username = &self.user_name,
            uid = self.uid.as_raw(),
            gid = self.gid.as_raw()
        )
    }

    pub fn etc_group(&self) -> String {
        format!("{groupname}:x:{gid}:",
            groupname = &self.group_name,
            gid = self.gid.as_raw()
        )
    }

    pub fn etc_sudoer(&self) -> String {
        format!("{username} ALL=(ALL) NOPASSWD: ALL",
            username = &self.user_name
        )
    }

    pub fn home_dir(&self) -> String {
        format!("/home/{username}",
            username = &self.user_name
        )
    }

    pub fn from_id(id: &str) -> Option<User> {
        let re = Regex::new(r"^uid=([0-9]+)\((\w*)\) gid=([0-9]+)\((\w*)\) .*$").unwrap();
        if let Some(caps) = re.captures(id) {
            Some(User {
                user_name: caps.get(2).unwrap().as_str().to_owned(),
                group_name: caps.get(4).unwrap().as_str().to_owned(),
                uid: nix::unistd::Uid::from_raw(caps.get(2).unwrap().as_str().parse().unwrap()),
                gid: nix::unistd::Gid::from_raw(caps.get(2).unwrap().as_str().parse().unwrap())
            })
        }
        else { None }
    }

    pub fn su(&self) -> Result<(), Error> {
        if self.uid.as_raw() != ROOT_UID {
            match mkuser(&self) {
                Ok(()) => {
                    let user_name: &str = &self.user_name;
                    let ret_su = unsafe { _su(std::ffi::CString::new(user_name).unwrap().into_raw()) };
                    if ret_su == 0 { Ok(()) }
                    else { Err(Error::NixError(nix::Error::last())) }
                }
                Err(e) => Err(Error::IOError(e))
            }
        }
        else { Ok(()) }
    }
}

pub enum Error {
    IOError(std::io::Error),
    NixError(nix::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IOError(e) => write!(f, "{}", e),
            Error::NixError(e) => write!(f, "{}", e),
        }
    }
}

fn mkuser(user: &User) -> Result<(), std::io::Error> {
    {
        let mut etc_passwd = OpenOptions::new().append(true).open("/etc/passwd")?;
        writeln!(etc_passwd, "{}", user.etc_passwd())?;
    }
    {
        let mut etc_group = OpenOptions::new().append(true).open("/etc/group")?;
        writeln!(etc_group, "{}", user.etc_group())?;
    }
    {
        let ref fsudoer = format!("/etc/sudoers.d/${username}", username = &user.user_name);
        let mut etc_group = OpenOptions::new().write(true).truncate(true).open(fsudoer)?;
        writeln!(etc_group, "{}", user.etc_sudoer())?;
        let mut permissions = etc_group.metadata().unwrap().permissions();
        permissions.set_mode(0o440);
    }
    {
        let home: &str = &user.home_dir();
        if !Path::new(home).exists() {
            std::fs::create_dir(home)?;
        }
        match nix::unistd::chown(home, Some(user.uid), Some(user.gid)) {
            Ok(()) => Ok(()),
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))
        }

    }
}
