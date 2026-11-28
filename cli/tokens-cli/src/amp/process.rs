use std::io::{self, Read};
use std::process::{Child, ChildStdout, Command, ExitStatus};
use std::sync::atomic::{AtomicBool, Ordering};
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

pub(super) struct Supervisor {
    cleanup_failed: AtomicBool,
    // Restore Linux adoption before Signals can re-raise cancellation.
    #[cfg(target_os = "linux")]
    _subreaper: platform::Subreaper,
    #[cfg(unix)]
    signals: platform::Signals,
}

impl Supervisor {
    pub(super) fn new() -> io::Result<Self> {
        #[cfg(unix)]
        let signals = platform::Signals::new()?;
        #[cfg(target_os = "linux")]
        let subreaper = platform::Subreaper::new()?;
        Ok(Self {
            cleanup_failed: AtomicBool::new(false),
            #[cfg(target_os = "linux")]
            _subreaper: subreaper,
            #[cfg(unix)]
            signals,
        })
    }

    pub(super) fn stopped(&self) -> bool {
        #[cfg(unix)]
        if self.signals.cancelled() {
            return true;
        }
        self.cleanup_failed.load(Ordering::Acquire)
    }

    pub(super) fn bounded_output(
        &self,
        command: &mut Command,
        timeout: Duration,
        limit: usize,
    ) -> Result<Vec<u8>, CaptureError> {
        #[cfg(target_os = "linux")]
        let group_ids = platform::lock_group_ids().map_err(|_| CaptureError::Run)?;
        if self.stopped() {
            return Err(CaptureError::Run);
        }
        let deadline = Instant::now() + timeout;
        let (mut child, mut tree) = platform::spawn(command).map_err(|_| CaptureError::Run)?;
        #[cfg(target_os = "linux")]
        drop(group_ids);
        let mut stdout = child.stdout.take();
        let result = capture(self, &mut child, &mut stdout, deadline, limit);
        drop(stdout);

        // Even a successful wrapper can leave descendants that closed stdout.
        if tree.cleanup(&mut child).is_err() {
            // An unconfirmed slot must not start another Amp command.
            self.cleanup_failed.store(true, Ordering::Release);
            return Err(CaptureError::Run);
        }
        result
    }
}

fn capture(
    supervisor: &Supervisor,
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
        if supervisor.stopped() {
            return Err(CaptureError::Run);
        }
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
    use std::mem::MaybeUninit;
    use std::os::fd::AsRawFd;
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::ptr::{null, null_mut};
    use std::sync::atomic::AtomicI32;
    use std::sync::{Mutex, MutexGuard};

    const CANCEL_SIGNALS: [libc::c_int; 4] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGQUIT];
    static SIGNAL_SCOPE: Mutex<()> = Mutex::new(());
    static CANCELLED_BY: AtomicI32 = AtomicI32::new(0);

    #[cfg(target_os = "linux")]
    static GROUP_IDS: Mutex<()> = Mutex::new(());

    #[cfg(target_os = "linux")]
    pub(super) fn lock_group_ids() -> io::Result<MutexGuard<'static, ()>> {
        // A recycled PGID must not select a new Amp child during a group wait.
        let mut guard = None;
        wait_for_exit(|| match GROUP_IDS.try_lock() {
            Ok(locked) => {
                guard = Some(locked);
                Ok(true)
            }
            Err(std::sync::TryLockError::WouldBlock) => Ok(false),
            Err(std::sync::TryLockError::Poisoned(_)) => {
                Err(io::Error::other("Amp process group lock poisoned"))
            }
        })?;
        Ok(guard.expect("success acquires the group lock"))
    }

    // Linux adoption is process-wide and restoring the flag cannot undo an
    // adoption. Amp sync has no other child trees or waiters in this CLI;
    // `serve` refreshes through a separate, short-lived submit process.
    #[cfg(target_os = "linux")]
    pub(super) struct Subreaper(bool);

    #[cfg(target_os = "linux")]
    impl Subreaper {
        pub(super) fn new() -> io::Result<Self> {
            let mut enabled: libc::c_int = 0;
            let unused: libc::c_ulong = 0;
            // SAFETY: PR_GET_CHILD_SUBREAPER writes an int; unused arguments are zero.
            if unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &raw mut enabled,
                    unused,
                    unused,
                    unused,
                )
            } == -1
            {
                return Err(io::Error::last_os_error());
            }
            if enabled == 0 {
                set_child_subreaper(true)?;
            }
            Ok(Self(enabled == 0))
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for Subreaper {
        fn drop(&mut self) {
            if !self.0 {
                return;
            }
            if let Err(error) = set_child_subreaper(false) {
                eprintln!("Could not restore the Amp child subreaper setting: {error}");
                std::process::exit(1);
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn set_child_subreaper(enabled: bool) -> io::Result<()> {
        let unused: libc::c_ulong = 0;
        // SAFETY: PR_SET_CHILD_SUBREAPER takes a 0 or 1 and three unused arguments.
        if unsafe {
            libc::prctl(
                libc::PR_SET_CHILD_SUBREAPER,
                enabled as libc::c_ulong,
                unused,
                unused,
                unused,
            )
        } == -1
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    pub(super) struct Signals {
        previous: [Option<libc::sigaction>; CANCEL_SIGNALS.len()],
        _exclusive: MutexGuard<'static, ()>,
    }

    impl Signals {
        pub(super) fn new() -> io::Result<Self> {
            let exclusive = SIGNAL_SCOPE.lock().unwrap_or_else(|error| error.into_inner());
            let child_action = signal_action(libc::SIGCHLD)?;
            if child_action.sa_sigaction != libc::SIG_DFL
                || child_action.sa_flags & libc::SA_NOCLDWAIT != 0
            {
                // The group leader must not be reaped outside its Child handle.
                return Err(io::ErrorKind::Unsupported.into());
            }

            CANCELLED_BY.store(0, Ordering::Relaxed);
            let mut signals = Self {
                previous: [None; CANCEL_SIGNALS.len()],
                _exclusive: exclusive,
            };
            // SAFETY: The initialized handler, flags and mask are valid for sigaction.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            action.sa_sigaction = note_cancellation as *const () as libc::sighandler_t;
            action.sa_flags = libc::SA_RESTART;
            unsafe { libc::sigemptyset(&mut action.sa_mask) };
            for (signal, previous) in CANCEL_SIGNALS.into_iter().zip(&mut signals.previous) {
                let old = signal_action(signal)?;
                if old.sa_sigaction != libc::SIG_DFL {
                    continue; // Respect inherited SIG_IGN and preexisting handlers.
                }
                // SAFETY: This scope exclusively owns the CLI's Amp signal registrations.
                if unsafe { libc::sigaction(signal, &action, null_mut()) } == -1 {
                    return Err(io::Error::last_os_error());
                }
                *previous = Some(old);
            }
            Ok(signals)
        }

        pub(super) fn cancelled(&self) -> bool {
            CANCELLED_BY.load(Ordering::Relaxed) != 0
        }
    }

    impl Drop for Signals {
        fn drop(&mut self) {
            for (signal, previous) in CANCEL_SIGNALS.into_iter().zip(&self.previous) {
                if let Some(previous) = previous {
                    // SAFETY: Previously read disposition for this same signal.
                    if unsafe { libc::sigaction(signal, previous, null_mut()) } == -1 {
                        // Do not keep running with a stale process-wide handler.
                        std::process::exit(1);
                    }
                }
            }
            let signal = CANCELLED_BY.swap(0, Ordering::Relaxed);
            if signal != 0 {
                // All export workers have joined. Restore ordinary signal exit semantics.
                // SAFETY: We unblock only the recorded signal, now at its default disposition.
                unsafe {
                    let mut mask = MaybeUninit::uninit();
                    libc::sigemptyset(mask.as_mut_ptr());
                    libc::sigaddset(mask.as_mut_ptr(), signal);
                    libc::pthread_sigmask(libc::SIG_UNBLOCK, mask.as_ptr(), null_mut());
                    libc::raise(signal);
                }
                std::process::exit(128 + signal);
            }
        }
    }

    extern "C" fn note_cancellation(signal: libc::c_int) {
        let _ = CANCELLED_BY.compare_exchange(0, signal, Ordering::Relaxed, Ordering::Relaxed);
    }

    fn signal_action(signal: libc::c_int) -> io::Result<libc::sigaction> {
        let mut action = MaybeUninit::uninit();
        // SAFETY: A null input queries the old action into valid output storage.
        if unsafe { libc::sigaction(signal, null(), action.as_mut_ptr()) } == -1 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: sigaction initialized the output.
        Ok(unsafe { action.assume_init() })
    }

    pub(super) struct Tree(Option<libc::pid_t>);

    pub(super) fn spawn(command: &mut Command) -> io::Result<(Child, Tree)> {
        let child = command.process_group(0).spawn()?;
        let tree = Tree(Some(child.id() as libc::pid_t));
        Ok((child, tree))
    }

    impl Tree {
        #[cfg(target_vendor = "apple")]
        pub(super) fn cleanup(&mut self, child: &mut Child) -> io::Result<()> {
            let group = self.0.expect("cleanup runs once");
            let idle = wait_for_exit(|| {
                signal_group(group)?;
                Ok(exit_status(child)?.is_some() && group_is_idle(group)?)
            });
            // Never signal the numeric group ID again once the leader can be reaped.
            self.0 = None;
            let reaped = child.try_wait();
            idle?;
            reaped?.ok_or(io::ErrorKind::TimedOut)?;
            Ok(())
        }

        #[cfg(target_os = "linux")]
        pub(super) fn cleanup(&mut self, child: &mut Child) -> io::Result<()> {
            let group = self.0.expect("cleanup runs once");
            wait_for_exit(|| {
                // Do not signal if the kernel no longer recognizes our leader.
                let exited = exit_status(child).inspect_err(|_| self.0 = None)?.is_some();
                signal_group(group)?;
                Ok(exited)
            })?;
            let group_ids = lock_group_ids();
            // Reap the leader by its handle even if the group lock timed out.
            // Once the leader can be reaped, never signal the numeric ID again.
            self.0 = None;
            child.try_wait()?.ok_or(io::ErrorKind::TimedOut)?;
            let _group_ids = group_ids?;
            reap_group(group)
        }

        #[cfg(all(not(target_vendor = "apple"), not(target_os = "linux")))]
        pub(super) fn cleanup(&mut self, child: &mut Child) -> io::Result<()> {
            let group = self.0.expect("cleanup runs once");
            wait_for_exit(|| {
                signal_group(group)?;
                Ok(exit_status(child)?.is_some())
            })?;
            // The leader pinned the group ID through the last SIGKILL. Reap
            // it, then only probe: even if the ID is reused, no new process
            // can receive a terminating signal from this supervisor.
            self.0 = None;
            child.wait()?;
            wait_for_exit(|| group_is_absent(group))
        }
    }

    impl Drop for Tree {
        fn drop(&mut self) {
            if let Some(group) = self.0 {
                let _ = signal_group(group);
            }
        }
    }

    fn signal_group(group: libc::pid_t) -> io::Result<()> {
        // SAFETY: The owned leader has not been reaped, so its PID still pins the group ID.
        if unsafe { libc::killpg(group, libc::SIGKILL) } == -1 {
            let error = io::Error::last_os_error();
            // Darwin can report EPERM for a zombie-only group. Neither error proves idleness.
            if !matches!(error.raw_os_error(), Some(libc::ESRCH | libc::EPERM)) {
                return Err(error);
            }
        }
        Ok(())
    }

    #[cfg(not(target_vendor = "apple"))]
    fn group_is_absent(group: libc::pid_t) -> io::Result<bool> {
        // SAFETY: signal 0 asks the kernel whether the group exists.
        if unsafe { libc::killpg(group, 0) } == 0 {
            return Ok(false);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(true)
        } else {
            Err(error)
        }
    }

    #[cfg(target_os = "linux")]
    fn reap_group(group: libc::pid_t) -> io::Result<()> {
        let deadline = Instant::now() + REAP_TIMEOUT;
        loop {
            // SAFETY: The negative PID selects only our children in this Amp group.
            let pid = unsafe { libc::waitpid(-group, null_mut(), libc::WNOHANG) };
            if pid == -1 {
                let error = io::Error::last_os_error();
                match error.raw_os_error() {
                    // ECHILD does not rule out a group member we do not own.
                    Some(libc::ECHILD) if group_is_absent(group)? => return Ok(()),
                    Some(libc::ECHILD | libc::EINTR) => {}
                    _ => return Err(error),
                }
            }
            let delay = POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now()));
            if delay.is_zero() {
                return Err(io::ErrorKind::TimedOut.into());
            }
            if pid <= 0 {
                thread::sleep(delay);
            }
        }
    }


    #[cfg(target_vendor = "apple")]
    fn group_is_idle(group: libc::pid_t) -> io::Result<bool> {
        let mut pids = [0; 2];
        // SAFETY: The writable PID buffer is sized in bytes. Clear errno to distinguish
        // an empty list from an error; both can return zero.
        let count = unsafe {
            *libc::__error() = 0;
            libc::proc_listpgrppids(
                group,
                pids.as_mut_ptr().cast(),
                std::mem::size_of_val(&pids) as libc::c_int,
            )
        };
        if count <= 0 {
            let error = io::Error::last_os_error();
            return match error.raw_os_error() {
                Some(libc::ESRCH) => Ok(true),
                Some(0) if count == 0 => Ok(true),
                _ => Err(error),
            };
        }
        // Trust only the snapshot, not separate PID queries that could miss a
        // fork between the snapshot and the query. Two slots expose any member
        // besides the pinned leader even if the snapshot is truncated.
        // If a zombie is still listed, the reap deadline bounds the wait.
        Ok(count == 1 && pids[0] == group)
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

    fn poll_stdout(stdout: &ChildStdout, delay: Duration) -> io::Result<()> {
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
        Ok(())
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

        pub(super) fn cleanup(&mut self, child: &mut Child) -> io::Result<()> {
            // SAFETY: The non-inheritable, owned job handle is still live.
            check(unsafe { TerminateJobObject(self.0.as_raw_handle(), 1) })?;
            wait_for_exit(|| Ok(child.try_wait()?.is_some() && self.is_idle()?))
        }

        fn is_idle(&self) -> io::Result<bool> {
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
    #[cfg(target_os = "linux")]
    const ORPHAN_OUTPUT: &str = "Amp fixture output";
    #[cfg(unix)]
    const CANCEL_DIRS: [&str; 2] = ["first", "second"];

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
        #[cfg(target_os = "linux")]
        match mode.as_str() {
            "non-reaping-ancestor" => return non_reaping_ancestor(&dir),
            "capture-orphan" => return capture_orphan(&dir),
            "exit" => return,
            _ => {}
        }
        #[cfg(unix)]
        if mode == "supervisor" {
            for signal in [libc::SIGINT, libc::SIGTERM] {
                // SAFETY: Only this isolated fixture's dispositions are changed.
                assert_ne!(unsafe { libc::signal(signal, libc::SIG_DFL) }, libc::SIG_ERR);
            }
            let supervisor = Supervisor::new().unwrap();
            thread::scope(|scope| {
                for subdir in CANCEL_DIRS {
                    let dir = dir.join(subdir);
                    fs::create_dir(&dir).unwrap();
                    let supervisor = &supervisor;
                    scope.spawn(move || {
                        let mut command = fixture("wrapper", &dir);
                        let error = supervisor
                            .bounded_output(
                                command.stdout(Stdio::piped()),
                                Duration::from_secs(3),
                                4096,
                            )
                            .unwrap_err();
                        assert!(matches!(error, CaptureError::Run), "{error:?}");
                    });
                }
            });
            return;
        }
        let ready = dir.join("ready");
        if matches!(mode.as_str(), "wrapper" | "wrapper-closed") {
            #[cfg(windows)]
            if mode == "wrapper-closed" {
                use std::os::windows::io::AsRawHandle;
                use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE_FLAG_INHERIT};
                // NUL redirects the child's stdout, but does not make the wrapper's
                // original pipe handle non-inheritable. Model a genuinely closed writer.
                let stdout = io::stdout();
                assert_ne!(
                    unsafe { SetHandleInformation(stdout.as_raw_handle(), HANDLE_FLAG_INHERIT, 0) },
                    0,
                    "{}",
                    io::Error::last_os_error()
                );
            }
            fixture("hold", &dir)
                .stdout(if mode == "wrapper" {
                    Stdio::inherit()
                } else {
                    Stdio::null()
                })
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while !ready.exists() {
                assert!(Instant::now() < deadline, "descendant did not start");
                thread::sleep(POLL_INTERVAL);
            }
            fs::write(dir.join("wrapper-exiting"), []).unwrap();
            #[cfg(target_os = "linux")]
            if mode == "wrapper-closed" {
                println!("{ORPHAN_OUTPUT}");
            }
            std::process::exit(0);
        }

        let lock = File::create(dir.join("alive")).unwrap();
        fs2::FileExt::lock_exclusive(&lock).unwrap();
        fs::write(ready, std::process::id().to_string()).unwrap();
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
        #[cfg(windows)]
        {
            let pid = fs::read_to_string(dir.join("ready")).unwrap().parse().unwrap();
            assert!(!crate::process_liveness::pid_is_alive(pid), "fixture {pid} is still running");
            // LockFileEx permits deferred unlock after termination. Check the
            // process above immediately, then wait only for its OS lock to clear.
            wait_for_exit(|| match fs2::FileExt::try_lock_exclusive(&lock) {
                Ok(()) => Ok(true),
                Err(err) if crate::commands::autosubmit::is_lock_contention(&err) => Ok(false),
                Err(err) => Err(err),
            })
            .expect("Windows did not release the terminated fixture's lock");
        }
        #[cfg(not(windows))]
        fs2::FileExt::try_lock_exclusive(&lock).expect("fixture still holds its lock");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_reaps_amp_orphan_with_nonreaping_ancestor() {
        let dir = tempfile::tempdir().unwrap();
        let status = fixture("non-reaping-ancestor", dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .unwrap();
        assert!(status.success(), "{status}");
        assert_stopped(dir.path());
        assert!(dir.path().join("wrapper-exiting").is_file());
    }

    #[cfg(target_os = "linux")]
    fn non_reaping_ancestor(dir: &Path) {
        let one: libc::c_ulong = 1;
        let unused: libc::c_ulong = 0;
        // SAFETY: Only this isolated fixture becomes a stand-in for PID 1.
        assert_eq!(
            unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, one, unused, unused, unused) },
            0
        );
        drop(Supervisor::new().unwrap());
        assert_eq!(subreaper_setting(), 1);
        let status = fixture("capture-orphan", dir)
            .stderr(Stdio::inherit())
            .status()
            .unwrap();

        // Do not reap while the capture runs. Clean up leftovers if it regresses.
        let mut adopted = false;
        wait_for_exit(|| {
            // SAFETY: This isolated fixture has no unrelated children.
            match unsafe { libc::waitpid(-1, std::ptr::null_mut(), libc::WNOHANG) } {
                -1 => {
                    let error = io::Error::last_os_error();
                    match error.raw_os_error() {
                        Some(libc::ECHILD) => Ok(true),
                        Some(libc::EINTR) => Ok(false),
                        _ => Err(error),
                    }
                }
                0 => Ok(false),
                _ => {
                    adopted = true;
                    Ok(false)
                }
            }
        })
        .unwrap();
        assert!(status.success(), "{status}");
        assert!(!adopted, "Amp orphan reached the non-reaping ancestor");
    }

    #[cfg(target_os = "linux")]
    fn capture_orphan(dir: &Path) {
        assert_eq!(subreaper_setting(), 0);
        let supervisor = Supervisor::new().unwrap();
        let mut unrelated = fixture("exit", dir).stdout(Stdio::null()).spawn().unwrap();
        wait_for_exit(|| Ok(platform::exit_status(&mut unrelated)?.is_some())).unwrap();
        let output = supervisor
            .bounded_output(
                fixture("wrapper-closed", dir).stdout(Stdio::piped()),
                Duration::from_secs(3),
                4096,
            )
            .unwrap();
        assert!(std::str::from_utf8(&output).unwrap().contains(ORPHAN_OUTPUT));
        assert!(unrelated.wait().unwrap().success());
        drop(supervisor);
        assert_eq!(subreaper_setting(), 0);
    }

    #[cfg(target_os = "linux")]
    fn subreaper_setting() -> libc::c_int {
        let mut enabled: libc::c_int = 0;
        let unused: libc::c_ulong = 0;
        // SAFETY: PR_GET_CHILD_SUBREAPER writes into this int.
        assert_eq!(
            unsafe {
                libc::prctl(
                    libc::PR_GET_CHILD_SUBREAPER,
                    &raw mut enabled,
                    unused,
                    unused,
                    unused,
                )
            },
            0
        );
        enabled
    }

    #[test]
    fn timeout_stops_child_and_inherited_stdout_writer() {
        let supervisor = Supervisor::new().unwrap();
        for mode in ["hold", "wrapper"] {
            let dir = tempfile::tempdir().unwrap();
            let mut command = fixture(mode, dir.path());
            let started = Instant::now();
            let error = supervisor
                .bounded_output(command.stdout(Stdio::piped()), Duration::from_millis(750), 4096)
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
        let supervisor = Supervisor::new().unwrap();
        let started = Instant::now();
        let error = supervisor
            .bounded_output(command.stdout(Stdio::piped()), Duration::from_secs(3), 4096)
            .unwrap_err();
        assert!(matches!(error, CaptureError::OutputTooLarge), "{error:?}");
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_stopped(dir.path());
    }

    #[test]
    fn closed_stdout_does_not_release_a_descendant_holding_a_lock() {
        let dir = tempfile::tempdir().unwrap();
        let supervisor = Supervisor::new().unwrap();
        let mut command = fixture("wrapper-closed", dir.path());
        supervisor
            .bounded_output(command.stdout(Stdio::piped()), Duration::from_secs(3), 4096)
            .unwrap();
        assert_stopped(dir.path());
        assert!(dir.path().join("wrapper-exiting").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn external_cancellation_reaps_all_active_groups_before_exit() {
        use std::os::unix::process::{CommandExt, ExitStatusExt};

        for signal in [libc::SIGINT, libc::SIGTERM] {
            let dir = tempfile::tempdir().unwrap();
            let mut child = fixture("supervisor", dir.path())
                .process_group(0)
                .stdout(Stdio::null())
                .spawn()
                .unwrap();
            let outcome = (|| -> io::Result<ExitStatus> {
                wait_for_exit(|| {
                    Ok(CANCEL_DIRS
                        .iter()
                        .all(|subdir| dir.path().join(subdir).join("ready").is_file()))
                })?;
                // SAFETY: This targets only the unreaped fixture's private foreground group.
                if unsafe { libc::killpg(child.id() as libc::pid_t, signal) } == -1 {
                    return Err(io::Error::last_os_error());
                }
                let mut status = None;
                wait_for_exit(|| {
                    status = child.try_wait()?;
                    Ok(status.is_some())
                })?;
                Ok(status.unwrap())
            })();
            let status = outcome.unwrap_or_else(|error| {
                let _ = child.kill();
                let _ = child.wait();
                panic!("isolated supervisor did not stop: {error}");
            });
            assert_eq!(status.signal(), Some(signal));
            for subdir in CANCEL_DIRS {
                assert_stopped(&dir.path().join(subdir));
            }
        }
    }
}
