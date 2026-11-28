use std::io::{self, Read};
use std::process::{Child, ChildStdout, Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);
const REAP_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub(super) enum CaptureError {
    Run,
    Stdout,
    TimedOut,
    OutputTooLarge,
    Exit(ExitStatus),
}

pub(super) fn bounded_output(
    command: &mut Command,
    timeout: Duration,
    limit: usize,
) -> Result<Vec<u8>, CaptureError> {
    let deadline = Instant::now() + timeout;
    let (mut child, mut tree) = platform::spawn(command).map_err(|_| CaptureError::Run)?;
    let mut stdout = child.stdout.take();
    let result = capture(&mut child, &mut stdout, deadline, limit);

    // Even a successful wrapper can leave descendants that closed stdout.
    let stopped = tree.terminate(&mut child).and_then(|()| {
        wait_for_exit(|| Ok(child.try_wait()?.is_some() && tree.is_idle(stdout.as_ref())?))
    });
    stopped.map_err(|_| CaptureError::Run)?;
    result
}

fn capture(
    child: &mut Child,
    stdout: &mut Option<ChildStdout>,
    deadline: Instant,
    limit: usize,
) -> Result<Vec<u8>, CaptureError> {
    platform::prepare_stdout(stdout.as_ref().ok_or(CaptureError::Stdout)?)
        .map_err(|_| CaptureError::Stdout)?;
    let mut output = Vec::new();
    let mut chunk = [0; 8192];
    let mut status = None;

    // Nonblocking reads keep the same deadline for the process and every
    // inherited writer, without a reader thread to cancel or join.
    loop {
        if Instant::now() >= deadline {
            return Err(CaptureError::TimedOut);
        }
        if let Some(pipe) = stdout.as_mut() {
            let remaining = limit - output.len();
            let read_len = chunk.len().min(remaining.saturating_add(1));
            match platform::read_stdout(pipe, &mut chunk[..read_len]) {
                Ok(0) => *stdout = None,
                Ok(n) if n > remaining => return Err(CaptureError::OutputTooLarge),
                Ok(n) => {
                    let len = output.len() + n;
                    if len > output.capacity() {
                        let capacity = (output.capacity() * 2).max(len).min(limit);
                        output
                            .try_reserve_exact(capacity - output.len())
                            .map_err(|_| CaptureError::Stdout)?;
                    }
                    output.extend_from_slice(&chunk[..n]);
                    continue;
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => return Err(CaptureError::Stdout),
            }
        }
        if status.is_none() {
            status = platform::exit_status(child).map_err(|_| CaptureError::Run)?;
        }
        if let Some(status) = status {
            if !status.success() {
                return Err(CaptureError::Exit(status));
            }
            if stdout.is_none() {
                return Ok(output);
            }
        }
        let delay = POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now()));
        platform::wait_for_stdout(stdout.as_ref(), delay).map_err(|_| CaptureError::Stdout)?;
    }
}

fn wait_for_exit(mut done: impl FnMut() -> io::Result<bool>) -> io::Result<()> {
    let deadline = Instant::now() + REAP_TIMEOUT;
    while !done()? {
        let delay = POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now()));
        if delay.is_zero() {
            return Err(io::ErrorKind::TimedOut.into());
        }
        thread::sleep(delay);
    }
    Ok(())
}

#[cfg(unix)]
mod platform {
    use super::*;
    use std::os::fd::AsRawFd;
    use std::os::unix::process::{CommandExt, ExitStatusExt};

    pub(super) struct Tree(Option<libc::pid_t>);

    pub(super) fn spawn(command: &mut Command) -> io::Result<(Child, Tree)> {
        let child = command.process_group(0).spawn()?;
        let tree = Tree(Some(child.id() as libc::pid_t));
        Ok((child, tree))
    }

    impl Tree {
        pub(super) fn terminate(&mut self, child: &mut Child) -> io::Result<()> {
            match self.signal_group() {
                Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                    // Darwin returns EPERM if only the zombie leader remains.
                    // After reaping, probe for absence, but never signal a pid
                    // that the OS is now free to reuse.
                    if child.try_wait()?.is_some() {
                        if let Some(group) = self.0.take() {
                            // SAFETY: Signal 0 checks only whether the group exists.
                            if unsafe { libc::killpg(group, 0) } == -1
                                && io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
                            {
                                return Ok(());
                            }
                        }
                    }
                    Err(err)
                }
                result => result,
            }
        }

        fn signal_group(&mut self) -> io::Result<()> {
            if let Some(group) = self.0 {
                // SAFETY: The group is private to the command. The leader has
                // not been reaped, so its pid cannot identify another group.
                if unsafe { libc::killpg(group, libc::SIGKILL) } == -1 {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() != Some(libc::ESRCH) {
                        return Err(error);
                    }
                }
                self.0 = None;
            }
            Ok(())
        }

        pub(super) fn is_idle(&self, stdout: Option<&ChildStdout>) -> io::Result<bool> {
            // HUP confirms all inherited writers closed, even with unread bytes.
            match stdout {
                Some(pipe) => Ok(poll_stdout(pipe, Duration::ZERO)? & libc::POLLHUP != 0),
                None => Ok(true),
            }
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            let _ = self.signal_group();
        }
    }

    pub(super) fn prepare_stdout(stdout: &ChildStdout) -> io::Result<()> {
        let fd = stdout.as_raw_fd();
        // SAFETY: The read end is open and both fcntl calls use the required arguments.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub(super) fn read_stdout(stdout: &mut ChildStdout, bytes: &mut [u8]) -> io::Result<usize> {
        stdout.read(bytes)
    }

    pub(super) fn exit_status(child: &mut Child) -> io::Result<Option<ExitStatus>> {
        // SAFETY: A zeroed siginfo_t is a valid output buffer. WNOWAIT observes
        // the exit without freeing the group leader's pid before killpg.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                child.id() as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result == -1 {
            let error = io::Error::last_os_error();
            return if error.kind() == io::ErrorKind::Interrupted {
                Ok(None)
            } else {
                Err(error)
            };
        }
        // SAFETY: A successful waitid filled the child fields, or left them zero.
        if unsafe { info.si_pid() } == 0 {
            return Ok(None);
        }
        let status = unsafe { info.si_status() };
        let raw = match info.si_code {
            libc::CLD_EXITED => status << 8,
            libc::CLD_KILLED => status,
            libc::CLD_DUMPED => status | 0x80,
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };
        Ok(Some(ExitStatus::from_raw(raw)))
    }

    pub(super) fn wait_for_stdout(stdout: Option<&ChildStdout>, delay: Duration) -> io::Result<()> {
        if let Some(pipe) = stdout {
            poll_stdout(pipe, delay)?;
        } else {
            thread::sleep(delay);
        }
        Ok(())
    }

    fn poll_stdout(stdout: &ChildStdout, delay: Duration) -> io::Result<libc::c_short> {
        let mut fd = libc::pollfd {
            fd: stdout.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: fd points to one live descriptor; delay is at most one poll interval.
        if unsafe { libc::poll(&mut fd, 1, delay.as_millis() as libc::c_int) } == -1 {
            let error = io::Error::last_os_error();
            if error.kind() != io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
        Ok(fd.revents)
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::mem::size_of;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::os::windows::process::CommandExt;
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{
        ERROR_BROKEN_PIPE, ERROR_NO_DATA, ERROR_PIPE_NOT_CONNECTED, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation,
        JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
        TerminateJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows_sys::Win32::System::Pipes::PeekNamedPipe;
    use windows_sys::Win32::System::Threading::{
        OpenThread, ResumeThread, CREATE_SUSPENDED, THREAD_SUSPEND_RESUME,
    };

    pub(super) struct Tree(OwnedHandle);

    pub(super) fn spawn(command: &mut Command) -> io::Result<(Child, Tree)> {
        let tree = Tree::new()?;
        // Keep std's .cmd handling, but give Amp no chance to spawn outside the job.
        let mut child = command.creation_flags(CREATE_SUSPENDED).spawn()?;
        // SAFETY: Both handles are live and the process's first thread is suspended.
        let assigned = check(unsafe {
            AssignProcessToJobObject(tree.0.as_raw_handle(), child.as_raw_handle())
        });
        if let Err(error) = assigned.and_then(|()| resume_main_thread(child.id())) {
            let _ = child.kill();
            let _ = wait_for_exit(|| Ok(child.try_wait()?.is_some()));
            return Err(error);
        }
        Ok((child, tree))
    }

    impl Tree {
        fn new() -> io::Result<Self> {
            // SAFETY: Null attributes make this unnamed job handle non-inheritable.
            let raw = unsafe { CreateJobObjectW(null(), null()) };
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = unsafe { OwnedHandle::from_raw_handle(raw) };
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            // SAFETY: The job is live and `limits` matches the information class.
            check(unsafe {
                SetInformationJobObject(
                    job.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            })?;
            Ok(Self(job))
        }

        pub(super) fn terminate(&mut self, _child: &mut Child) -> io::Result<()> {
            // SAFETY: The non-inheritable, owned job handle is still live.
            check(unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) })
        }

        pub(super) fn is_idle(&self, _stdout: Option<&ChildStdout>) -> io::Result<bool> {
            let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { std::mem::zeroed() };
            // SAFETY: The job is live and `accounting` matches the information class.
            check(unsafe {
                QueryInformationJobObject(
                    self.0.as_raw_handle(),
                    JobObjectBasicAccountingInformation,
                    (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            })?;
            Ok(accounting.ActiveProcesses == 0)
        }
    }

    fn resume_main_thread(pid: u32) -> io::Result<()> {
        // std does not expose the primary thread handle on stable Rust. A new
        // suspended process has not run code to start any other threads.
        // SAFETY: The snapshot flags request thread records only.
        let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
        entry.dwSize = size_of::<THREADENTRY32>() as u32;
        // SAFETY: The snapshot handle and correctly sized output buffer are live.
        unsafe {
            check(Thread32First(snapshot.as_raw_handle(), &mut entry))?;
            while entry.th32OwnerProcessID != pid {
                check(Thread32Next(snapshot.as_raw_handle(), &mut entry))?;
            }
            let raw = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            let thread = OwnedHandle::from_raw_handle(raw);
            if ResumeThread(thread.as_raw_handle()) == u32::MAX {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }

    pub(super) fn prepare_stdout(_stdout: &ChildStdout) -> io::Result<()> {
        Ok(())
    }

    pub(super) fn read_stdout(stdout: &mut ChildStdout, bytes: &mut [u8]) -> io::Result<usize> {
        let mut available = 0;
        // SAFETY: stdout is a live pipe and `available` is the only output requested.
        let peeked = unsafe {
            PeekNamedPipe(
                stdout.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        };
        if peeked == 0 {
            let error = io::Error::last_os_error();
            return match error.raw_os_error().map(|code| code as u32) {
                Some(ERROR_BROKEN_PIPE | ERROR_NO_DATA | ERROR_PIPE_NOT_CONNECTED) => Ok(0),
                _ => Err(error),
            };
        }
        if available == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        // We are the only reader, so these bytes remain ready for ReadFile.
        let count = bytes.len().min(available as usize);
        stdout.read(&mut bytes[..count])
    }

    pub(super) fn exit_status(child: &mut Child) -> io::Result<Option<ExitStatus>> {
        child.try_wait()
    }

    pub(super) fn wait_for_stdout(_stdout: Option<&ChildStdout>, delay: Duration) -> io::Result<()> {
        thread::sleep(delay);
        Ok(())
    }

    fn check(result: i32) -> io::Result<()> {
        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::process::Stdio;

    const MODE: &str = "TOKENS_TEST_AMP_PROCESS_MODE";
    const DIR: &str = "TOKENS_TEST_AMP_PROCESS_DIR";

    fn fixture(mode: &str, dir: &Path) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "amp::process::tests::fixture_process",
                "--ignored",
                "--nocapture",
            ])
            .env_clear()
            .env(MODE, mode)
            .env(DIR, dir)
            .stdin(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        if let Some(root) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", root);
        }
        command
    }

    #[test]
    #[ignore = "subprocess fixture"]
    #[allow(clippy::zombie_processes, reason = "the supervisor must stop this descendant")]
    fn fixture_process() {
        let mode = std::env::var(MODE).unwrap();
        let dir = std::path::PathBuf::from(std::env::var_os(DIR).unwrap());
        let ready = dir.join("ready");
        if mode == "wrapper" {
            fixture("hold", &dir)
                .stdout(Stdio::inherit())
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while !ready.exists() {
                assert!(Instant::now() < deadline, "descendant did not start");
                thread::sleep(POLL_INTERVAL);
            }
            fs::write(dir.join("wrapper-exiting"), []).unwrap();
            std::process::exit(0);
        }

        let lock = File::create(dir.join("alive")).unwrap();
        fs2::FileExt::lock_exclusive(&lock).unwrap();
        fs::write(ready, []).unwrap();
        if mode == "oversize" {
            let mut stdout = io::stdout().lock();
            let _ = stdout.write_all(&[b'x'; 8192]).and_then(|()| stdout.flush());
        }
        thread::sleep(Duration::from_secs(4));
        drop(lock);
    }

    fn assert_stopped(dir: &Path) {
        assert!(dir.join("ready").is_file(), "fixture did not acquire its lock");
        let lock = File::options()
            .read(true)
            .write(true)
            .open(dir.join("alive"))
            .unwrap();
        fs2::FileExt::try_lock_exclusive(&lock).expect("fixture still holds its lock");
    }

    #[test]
    fn timeout_stops_child_and_inherited_stdout_writer() {
        for mode in ["hold", "wrapper"] {
            let dir = tempfile::tempdir().unwrap();
            let mut command = fixture(mode, dir.path());
            let started = Instant::now();
            let error = bounded_output(
                command.stdout(Stdio::piped()),
                Duration::from_millis(750),
                4096,
            )
            .unwrap_err();
            assert!(matches!(error, CaptureError::TimedOut), "{error:?}");
            assert!(started.elapsed() < Duration::from_secs(3));
            if mode == "wrapper" {
                assert!(dir.path().join("wrapper-exiting").is_file());
            }
            assert_stopped(dir.path());
        }
    }

    #[test]
    fn stdout_overflow_stops_child_before_its_deadline() {
        let dir = tempfile::tempdir().unwrap();
        let mut command = fixture("oversize", dir.path());
        let started = Instant::now();
        let error = bounded_output(command.stdout(Stdio::piped()), Duration::from_secs(3), 4096)
            .unwrap_err();
        assert!(matches!(error, CaptureError::OutputTooLarge), "{error:?}");
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_stopped(dir.path());
    }
}
