use crate::ffi::{c_int, c_size_t, c_ssize_t, c_void};
use crate::fmt;
pub(crate) struct Stderr;

impl fmt::Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        stderr_write_all(s.as_bytes());
        Ok(())
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        fmt::write_inner(self, args)
    }
}

fn stderr_write_all(mut buf: &[u8]) {
    const EINTR: c_int = 4;
    while !buf.is_empty() {
        match stderr_write(buf) {
            Ok(0) => panic!("[WRITE_FMT] error: failed to write whole buffer to stderr"),
            Ok(n) => buf = &buf[n..],
            Err(e) if e == EINTR => {}
            Err(e) => panic!("[WRITE_FMT] error: writing {} bytes to stderr: errno {e}", buf.len()),
        }
    }
}

fn stderr_write(buf: &[u8]) -> Result<usize, i32> {
    const READ_LIMIT: usize = if cfg!(target_vendor = "apple") {
        c_int::MAX as usize - 1
    } else {
        c_ssize_t::MAX as usize
    };

    unsafe extern "C" {
        fn write(fd: c_int, buf: *const c_void, count: c_size_t) -> c_ssize_t;
    }

    let ret = unsafe { write(2, buf.as_ptr() as *const c_void, buf.len().min(READ_LIMIT)) };
    if ret == -1 { Err(errno()) } else { Ok(ret as usize) }
}

fn errno() -> i32 {
    unsafe extern "C" {
        #[cfg(not(any(target_os = "dragonfly", target_os = "vxworks", target_os = "rtems")))]
        #[cfg_attr(
            any(
                target_os = "linux",
                target_os = "emscripten",
                target_os = "fuchsia",
                target_os = "l4re",
                target_os = "hurd",
            ),
            link_name = "__errno_location"
        )]
        #[cfg_attr(
            any(
                target_os = "netbsd",
                target_os = "openbsd",
                target_os = "android",
                target_os = "redox",
                target_os = "nuttx",
                target_env = "newlib"
            ),
            link_name = "__errno"
        )]
        #[cfg_attr(any(target_os = "solaris", target_os = "illumos"), link_name = "___errno")]
        #[cfg_attr(target_os = "nto", link_name = "__get_errno_ptr")]
        #[cfg_attr(any(target_os = "freebsd", target_vendor = "apple"), link_name = "__error")]
        #[cfg_attr(target_os = "haiku", link_name = "_errnop")]
        #[cfg_attr(target_os = "aix", link_name = "_Errno")]
        fn errno_location() -> *mut c_int;
    }
    unsafe { (*errno_location()) as i32 }
}
