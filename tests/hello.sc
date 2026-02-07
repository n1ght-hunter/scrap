extern "C" {
    fn GetStdHandle(nStdHandle: usize) -> usize;
    fn WriteFile(hFile: usize, lpBuffer: String, nNumberOfBytesToWrite: usize,
                 lpNumberOfBytesWritten: usize, lpOverlapped: usize) -> usize;
    fn ExitProcess(exit_code: usize) -> !;
}

fn main() {
    let stdout: usize = GetStdHandle(4294967285);
    let result: usize = WriteFile(stdout, "Hello, World!\n", 14, 0, 0);
    ExitProcess(0);
}
