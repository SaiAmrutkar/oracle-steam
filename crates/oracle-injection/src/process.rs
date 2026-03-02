use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub name: String,
}

#[cfg(target_os = "windows")]
pub fn find_process(name: &str) -> Result<Process> {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot as isize == -1 {
            anyhow::bail!("Failed to create process snapshot");
        }

        let mut process_entry: PROCESSENTRY32 = std::mem::zeroed();
        process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut process_entry) != 0 {
            loop {
                let process_name = std::ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr())
                    .to_string_lossy()
                    .trim_end_matches('\0')
                    .to_string();

                if process_name.eq_ignore_ascii_case(name) {
                    CloseHandle(snapshot);
                    log::info!(
                        "✓ Found process: {} (PID {})",
                        process_name,
                        process_entry.th32ProcessID
                    );
                    return Ok(Process {
                        pid: process_entry.th32ProcessID,
                        name: process_name,
                    });
                }

                if Process32Next(snapshot, &mut process_entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        anyhow::bail!("Process not found: {}", name)
    }
}

#[cfg(target_os = "windows")]
pub fn list_processes() -> Result<Vec<Process>> {
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::tlhelp32::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    };

    let mut processes = Vec::new();

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot as isize == -1 {
            anyhow::bail!("Failed to create process snapshot");
        }

        let mut process_entry: PROCESSENTRY32 = std::mem::zeroed();
        process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(snapshot, &mut process_entry) != 0 {
            loop {
                let name = std::ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr())
                    .to_string_lossy()
                    .trim_end_matches('\0')
                    .to_string();

                processes.push(Process {
                    pid: process_entry.th32ProcessID,
                    name,
                });

                if Process32Next(snapshot, &mut process_entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
    }

    Ok(processes)
}

#[cfg(not(target_os = "windows"))]
pub fn find_process(_name: &str) -> Result<Process> {
    anyhow::bail!("Process enumeration only supported on Windows")
}

#[cfg(not(target_os = "windows"))]
pub fn list_processes() -> Result<Vec<Process>> {
    anyhow::bail!("Process enumeration only supported on Windows")
}
