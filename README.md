# LakeOS

LakeOS is a hobbyist OS written in Rust, featuring a L4-like microkernel and a userland similar to Barrelfish.

The kernel, similar to seL4, is a capability based micro kernel. The kernel tracks kernel objects as capabilities and expose a set of API to user space to securely manipulate kenrel objects.

The userland applications has a design similar to Barrelfish, featuring userlevel asynchronous IPC and self paging. Applications are also responsible for managing their own CSpace and VSpace, including allocating and installing page tables and frames.

Currently LakeOS runs on Raspberry Pi 3B+. The toolchain is based on Sergio's cs140e course project.

The kernel brings up a init thread after bootstrap. Init thread spawns other basic apps, including console driver, timer driver and shell.

All contributions are welcomed!

## Usage

The project requires:
- aarch64-none-elf binutils
- GNU make
- Reasonably recent Rust nightly toolchain, tested on `rustc 1.54.0-nightly (676ee1472 2021-05-06)`
- cargo components: `cargo-binutils`, `rust-src`
- QEMU for arm64: `qemu-system-aarch64`, make sure the version you use supports Raspberry Pi 3 by checking `qemu-system-aarch64 -machine help`

To build project, in kernel dir:
```
$ cd kernel
$ make
```

To run on QEMU:
```
$ cd kernel
$ make qemu
```

## Project Structure
- kernel: The L4-like micro kernel. 
- lib: Libraries used by kernel or apps. below are some important ones:
  - rustyl4api: API binding between kernel and user space. Including syscall, errno, constants, etc.
  - naive: The main runtime libraryi, providing APIs to basic OS services like allocator, RPC, file system, etc.
- userland:
  - init_thread: The first process brought up after kenel bootstrap. It spawns other processes from initfs, then works as some other servers what ought to be moved out in the future (VFS and physical memory allocator).
  - timer: The RPI3B system timer server.
  - console: The RPI3B UART console server.
  - shell: A simple shell implements a few simple commands (e.g. echo, ls, cd, cat, etc).

## Roadmap
### Kernel 
- [x] A basic kernel brings up init thread
- [x] Interupt
- [x] TLB and ASID handling
- [ ] Fault handler
- [ ] SMP
- [ ] Guarded page table based CSpace
- [ ] floating point register lazy store
- [ ] prioritized scheduler

### Drivers
- [x] GPIO and UART
- [x] Timer
- [ ] SD card
- [ ] NIC

### Basic System Service
- [x] IPC framework
- [x] VFS
- [x] Process
- [x] console server
- [x] async/await
- [ ] FAT32
- [ ] pager
- [ ] Network stack
- [ ] SMP multithreading
- [ ] libstd

### Userland
- [x] Shell
- [ ] basic network tools

## Credit
- [The seL4 MicroKernel](https://sel4.systems)
- [The Barrelfish OS](http://www.barrelfish.org/)
- [CS140e course page](https://cs140e.sergio.bz/)
- [CS140e implementation from jiegec](https://github.com/jiegec/cs140e)