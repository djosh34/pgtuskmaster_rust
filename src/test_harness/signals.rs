use std::io;

#[cfg(unix)]
pub(crate) fn send_signal(pid: u32, signal: i32) -> Result<(), io::Error> {
    let pid_i32 = i32::try_from(pid).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("pid out of range for pid_t: {pid}: {err}"),
        )
    })?;

    // SAFETY: libc::kill has no Rust safety contract; the pid and signal are validated inputs.
    let rc = unsafe { libc::kill(pid_i32, signal) };
    if rc == 0 {
        return Ok(());
    }

    let err = io::Error::last_os_error();
    match err.raw_os_error() {
        Some(code) if code == libc::ESRCH => Ok(()),
        Some(_) => Err(err),
        None => Err(err),
    }
}

#[cfg(unix)]
pub(crate) fn pid_exists(pid: u32) -> Result<bool, io::Error> {
    let pid_i32 = i32::try_from(pid).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("pid out of range for pid_t: {pid}: {err}"),
        )
    })?;

    // SAFETY: libc::kill has no Rust safety contract; the pid is validated input.
    let rc = unsafe { libc::kill(pid_i32, 0) };
    if rc == 0 {
        return Ok(true);
    }

    let err = io::Error::last_os_error();
    match err.raw_os_error() {
        Some(code) if code == libc::ESRCH => Ok(false),
        Some(code) if code == libc::EPERM => Ok(true),
        Some(_) => Err(err),
        None => Err(err),
    }
}

#[cfg(not(unix))]
pub(crate) fn send_signal(_pid: u32, _signal: i32) -> Result<(), io::Error> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "signals are only supported on unix",
    ))
}

#[cfg(not(unix))]
pub(crate) fn pid_exists(_pid: u32) -> Result<bool, io::Error> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "signals are only supported on unix",
    ))
}
