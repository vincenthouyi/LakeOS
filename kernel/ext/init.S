#define BIT(x) (1 << (x))

#define CONTROL_I 12 // Instruction access Cacheability control
#define CONTROL_C 2  // Cacheability control, for data accesses
#define CONTROL_M 0  // MMU enable
#define SCTLR_VALUE                     (BIT(CONTROL_I) | BIT(CONTROL_C) | BIT(CONTROL_M))

#define TCR_T0SZ                        ((64 - 48) << 0)
#define TCR_T1SZ                        ((64 - 48) << 16)
#define TCR_TG0_4K                      (0 << 14)
#define TCR_TG1_4K                      (2 << 30)
#define TCR_A1                          (0 << 22) // Use TTBR0_EL1.ASID as ASID
#define TCR_AS                          (1 << 36) // 16 bit ASID size
#define TCR_VALUE                       (TCR_T0SZ | TCR_T1SZ | TCR_TG0_4K | TCR_TG1_4K | TCR_AS | TCR_A1)

#define vector(handler) \
    .align 7; \
    b       #handler

#define kernel_enter \
    stp     x0,  x1,  [sp, #16 * 0]; \
    stp     x2,  x3,  [sp, #16 * 1]; \
    stp     x4,  x5,  [sp, #16 * 2]; \
    stp     x6,  x7,  [sp, #16 * 3]; \
    stp     x8,  x9,  [sp, #16 * 4]; \
    stp     x10, x11, [sp, #16 * 5]; \
    stp     x12, x13, [sp, #16 * 6]; \
    stp     x14, x15, [sp, #16 * 7]; \
    stp     x16, x17, [sp, #16 * 8]; \
    stp     x18, x19, [sp, #16 * 9]; \
    stp     x20, x21, [sp, #16 * 10]; \
    stp     x22, x23, [sp, #16 * 11]; \
    stp     x24, x25, [sp, #16 * 12]; \
    stp     x26, x27, [sp, #16 * 13]; \
    stp     x28, x29, [sp, #16 * 14]; \
    mrs     x21, sp_el0; \
    mrs     x22, elr_el1; \
    mrs     x23, spsr_el1; \
    stp     x30, x21, [sp, #16 * 15]; \
    stp     x22, x23, [sp, #16 * 16]; \

#define kernel_exit \
    ldp     x22, x23, [sp], #16; \
    ldp     x30, x21, [sp], #16; \
    msr     spsr_el1, x23; \
    msr     elr_el1, x22; \
    msr     sp_el0, x21; \
    ldp     x28, x29, [sp], #16; \
    ldp     x26, x27, [sp], #16; \
    ldp     x24, x25, [sp], #16; \
    ldp     x22, x23, [sp], #16; \
    ldp     x20, x21, [sp], #16; \
    ldp     x18, x19, [sp], #16; \
    ldp     x16, x17, [sp], #16; \
    ldp     x14, x15, [sp], #16; \
    ldp     x12, x13, [sp], #16; \
    ldp     x10, x11, [sp], #16; \
    ldp     x8, x9, [sp], #16; \
    ldp     x6, x7, [sp], #16; \
    ldp     x4, x5, [sp], #16; \
    ldp     x2, x3, [sp], #16; \
    ldp     x0, x1, [sp], #16; \

.section .boot.text.
.globl _start
_start:
    msr     daifset, 0xf
    mrs     x0, mpidr_el1        // check core id, only one core is used.
    mov     x1, #0xc1000000
    bic     x0, x0, x1
    cbz     x0, zero_bss
hang:
    b       hang

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, jump_to_el1 
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

jump_to_el1:
    ldr     x0, =kernel_stack + 4096
    msr     sp_el1, x0           // init the stack of el1

    mov     x0, #3 << 20
    msr     cpacr_el1, x0        // enable fp/simd at el1

    // initialize hcr_el2
    mov     x0, #(1 << 31)
    msr     hcr_el2, x0          // set el1 to 64 bit

    /* put spsr to a known state */
    mov     x0, #(15 << 6 | 0b01 << 2 | 1) // DAIF masked, EL1, SpSelx 
    msr     spsr_el2, x0

    /* set up exception handlers (guide: 10.4) */
    ldr     x2, =trap_vectors
    msr     VBAR_EL1, x2

    /* Translation Control Register */
    ldr     x4, =TCR_VALUE
    msr     tcr_el1, x4
    isb

    /* Initialize page tables */
    bl      init_cpu

    /* Initialize SCTLR_EL1 */
    ldr     x0, =SCTLR_VALUE
    msr     sctlr_el1, x0
    isb

    ldr     x0, =kmain
    msr     elr_el2, x0

    /* jump to kmain in higher half runing in el1 */
    eret

.align 11
.global trap_vectors
trap_vectors:
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)

    vector(sync_trap)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)

    vector(lower64_trap)
    vector(lower64_irq)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)

    vector(unknown_exception_handler)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)
    vector(unknown_exception_handler)

sync_trap:
    b sync_handler

lower64_trap:
    kernel_enter
    mov     x0, sp
    ldr     x2, =kernel_stack + 4096
    mov     sp, x2
    b lower64_sync_handler

lower64_irq:
    kernel_enter
    mov     x0, sp
    ldr     x2, =kernel_stack + 4096
    mov     sp, x2
    b lower64_irq_handler