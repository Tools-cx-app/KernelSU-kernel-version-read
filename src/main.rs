use libc::{c_int, c_void};

const CMD_GET_VERSION: c_int = 2;
const KERNEL_SU_OPTION: u32 = 0xDEADBEEF;

/// 执行内核SU控制命令
///
/// # 参数
/// - `cmd`: 执行的命令（CMD_* 常量）
/// - `arg1`: 命令参数1（可为空）
/// - `arg2`: 命令参数2（可为空）
///
/// # 返回值
/// 操作是否成功（检查内核返回的魔数）
fn ksuctl(cmd: c_int, arg1: *mut c_void, arg2: *mut c_void) -> bool {
    unsafe {
        let mut result: u32 = 0;
        // 调用 prctl 系统调用
        libc::syscall(
            libc::SYS_prctl,
            KERNEL_SU_OPTION as libc::c_ulong,
            cmd as libc::c_ulong,
            arg1 as libc::c_ulong,
            arg2 as libc::c_ulong,
            &mut result as *mut u32 as libc::c_ulong,
        );
        result == KERNEL_SU_OPTION
    }
}

/// 获取内核SU版本
///
/// # 返回值
/// 版本号（获取失败返回 -1）
fn get_version_code() -> c_int {
    let mut version: c_int = -1;
    ksuctl(
        CMD_GET_VERSION,
        &mut version as *mut c_int as *mut c_void,
        std::ptr::null_mut(),
    );
    version
}

fn main() {
    let version = get_version_code();
    println!("{version}");
}
