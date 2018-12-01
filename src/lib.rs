extern crate nix;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;

extern "C" {
    fn _su(user: *mut std::os::raw::c_char) -> i32;
}

pub struct User {
    user_name: String,
    group_name: String,
    uid: nix::unistd::Uid,
    gid: nix::unistd::Gid
}

impl User {
    fn etc_passwd(&self) -> String {
        format!("{username}:x:{uid}:{gid}:{username},,,:/home/{username}:/bin/bash",
            username = &self.user_name,
            uid = self.uid.as_raw(),
            gid = self.gid.as_raw()
        )
    }

    fn etc_group(&self) -> String {
        format!("{groupname}:x:{gid}:",
            groupname = &self.group_name,
            gid = self.gid.as_raw()
        )
    }

    fn etc_sudoer(&self) -> String {
        format!("{username} ALL=(ALL) NOPASSWD: ALL",
            username = &self.user_name
        )
    }

    fn home(&self) -> String {
        format!("/home/{username}",
            username = &self.user_name
        )
    }
}

pub fn su(user: &str) {
    unsafe { _su(std::ffi::CString::new(user).unwrap().into_raw()); }
}

pub fn mkuser(user: User) {
    if let Ok(mut etc_passwd) = OpenOptions::new().append(true).open("/etc/passwd") {
        writeln!(etc_passwd, "{}", user.etc_passwd()).expect("impersonate: mkuser: IO Error: /etc/passwd");
    }
    if let Ok(mut etc_group) = OpenOptions::new().append(true).open("/etc/group") {
        writeln!(etc_group, "{}", user.etc_group()).expect("impersonate: mkuser: IO Error: /etc/group");
    }
    let ref fsudoer = format!("/etc/sudoers.d/${username}", username = &user.user_name);
    if let Ok(mut etc_group) = OpenOptions::new().write(true).truncate(true).open(fsudoer) {
        writeln!(etc_group, "{}", user.etc_sudoer()).expect("impersonate: mkuser: IO Error: /etc/sudoers.d");
        let mut permissions = etc_group.metadata().unwrap().permissions();
        permissions.set_mode(0o440);
    }
    let home: &str = &user.home();
    #[allow(unused_must_use)]
    match std::fs::create_dir(home) {
        Ok(_) => {
            nix::unistd::chown(home, Some(user.uid), Some(user.gid));
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            nix::unistd::chown(home, Some(user.uid), Some(user.gid));
        }
        Err(_) => { }
    }
}
