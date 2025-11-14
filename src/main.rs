use std::fs;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::os::fd::RawFd;

use libc::{c_int, c_void};

const KSU_IOCTL_GET_INFO: u32 = 0x80004b02;
const KSU_INSTALL_MAGIC1: u32 = 0xDEADBEEF;
const KSU_INSTALL_MAGIC2: u32 = 0xCAFEBABE;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct GetInfoCmd {
    version: u32,
    flags: u32,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn scan_driver_fd() -> Option<RawFd> {
    let fd_dir = fs::read_dir("/proc/self/fd").ok()?;

    for entry in fd_dir.flatten() {
        if let Ok(fd_num) = entry.file_name().to_string_lossy().parse::<i32>() {
            let link_path = format!("/proc/self/fd/{}", fd_num);
            if let Ok(target) = fs::read_link(&link_path) {
                let target_str = target.to_string_lossy();
                if target_str.contains("[ksu_driver]") {
                    return Some(fd_num);
                }
            }
        }
    }

    None
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn init_driver_fd() -> Option<RawFd> {
    let fd = scan_driver_fd();
    if fd.is_none() {
        let mut fd = -1;
        unsafe {
            libc::syscall(
                libc::SYS_reboot,
                KSU_INSTALL_MAGIC1,
                KSU_INSTALL_MAGIC2,
                0,
                &mut fd,
            );
        };
        if fd >= 0 { Some(fd) } else { None }
    } else {
        fd
    }
}

// ioctl wrapper using libc
#[cfg(any(target_os = "linux", target_os = "android"))]
fn ksuctl<T>(request: u32, arg: *mut T) -> std::io::Result<i32> {
    use std::io;

    let fd = init_driver_fd().unwrap_or(-1);
    unsafe {
        #[cfg(not(target_env = "gnu"))]
        let ret = libc::ioctl(fd as libc::c_int, request as i32, arg);
        #[cfg(target_env = "gnu")]
        let ret = libc::ioctl(fd as libc::c_int, request as u64, arg);
        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret)
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_info() -> GetInfoCmd {
    let mut cmd = GetInfoCmd {
        version: 0,
        flags: 0,
    };
    let _ = ksuctl(KSU_IOCTL_GET_INFO, &mut cmd as *mut _);
    cmd
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_version_code() -> u32 {
    get_info().version
}

fn main() {
    let version = get_version_code();
    println!("{version}");
}
