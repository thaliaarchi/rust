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
        fn errno_location() -> *mut c_int;
    }

    let ret = unsafe { write(2, buf.as_ptr() as *const c_void, buf.len().min(READ_LIMIT)) };
    if ret == -1 {
        let errno = unsafe { (*errno_location()) as i32 };
        Err(errno)
    } else {
        Ok(ret as usize)
    }
}
