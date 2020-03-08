# LakeOS
A L4-like microkernel OS written in Rust

The kernel design is similar to seL4 and barrelfish, but has a lot of simplification. Currently it runs on Raspberry Pi 3B+ as well as QEMU. The toolchain is similar to Sergio's cs140e course.

So far the kernel only boots and brings up a init thread with a basic CSPace. Init thread can allocate objects on heap and build up its own virtual space. Init thread also runs a basic console server. But the server will be separated from init thread into a dedicated process.

The final target of this project is to implement a microkernel based containerization menchanism. The system will probably not be POSIX compatible.

Some code are taken from other open source projects, including Sergio's course code, jiegec's cs140e implementation, seL4's bootup code, and other Raspberry Pi tutorials. Will replace them gradually in the future...

All contribution is welcome!

# Usage

The project requires:
- aarch64-none-elf binutils
- GNU make

If run on QEMU:
- qemu

To build project, in kernel dir:
```
$ make
```

To run on QEMU:
```
$ make qemu
```

# Roadmap
## Kernel 
- [x] A basic kernel brings up init thread
- [ ] TLB ans ASID handling
- [ ] SMP
- [ ] Guarded page table based CSpace
- [ ] floating point register lazy store
- [ ] prioritized scheduler
- [ ] Interupt

## Drivers
- [ ] GPIO and UART
- [ ] NIC
- [ ] Timer
- [ ] SD card

## Basic System Service
- [ ] IPC framework
- [ ] VFS
- [ ] FAT32 (readonly)
- [ ] Process
- [ ] pager
- [ ] console server
- [ ] Network stack
- [ ] SMP multithreading
- [ ] async/await
- [ ] libstd

## Userland
- [ ] Shell
- [ ] basic network tools

## Containerization
TBD

