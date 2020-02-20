#ifndef OSLABPI_MMU_H
#define OSLABPI_MMU_H

#define BY2PG   4096
#define PDMAP   (BY2PG * 512 * 512) // size of whe whole page table
#define PDSHIFT 30
#define PMSHIFT 21
#define PGSHIFT 12

#define PDX(va) ((((unsigned long)(va)) >> PDSHIFT) & 0x1FF)
#define PMX(va) ((((unsigned long)(va)) >> PMSHIFT) & 0x1FF)
#define PTX(va) ((((unsigned long)(va)) >> PGSHIFT) & 0x1FF)

#define PTE_ADDR(pte) ((unsigned long)(pte) & 0xFFFFF000)

#define PPN(va) (((unsigned long)(va) & 0x0000007FFFFFFFFFUL) >> 12)
#define VPN(va) PPN(va)

#define KADDR(pa) ((unsigned long)(pa) | 0xFFFFFF8000000000UL)
#define PADDR(kva) ((unsigned long)(kva) & 0xFFFFFFFFUL)


/*  Kernel/Physical Memory Layout
 *
 *              +-------------+--------------------------------+  0x40001000
 *              |             | Interrupt Mask Control         |
 *              |  Device     +--------------------------------+  0x40000000
 *              |             | Conventional Device I/O        |
 *    P_LIMIT   +-------------+--------------------------------+  0x3F000000
 *              |              PAGED MEMORY                    |
 *              +-------------+--------------------------------+  0x01800000 <==P_RESERVED_TOP
 *              |             | Time Stack                     |
 *              +-------------+--------------------------------+  0x00B60000
 *              |  Kernel     | ENVS  ~ 0x00060000             |
 *              |  Data       +--------------------------------+  0x00B00000
 *              |  Structure  | PAGES ~ 0x00600000             |
 *              +-------------+--------------------------------+  0x00500000
 *              |   VM        | (Page frames used to init vm)  |
 *              |             | Kernel Page Directory          |
 *              +-------------+--------------------------------+  0x00200000
 *              |             | Kernel Text                    |     (Max kernel size 1.5MB)
 *  KERNEL_TEXT +-------------+--------------------------------+  0x00080000
 *  KSTACK _/   |             | Kernel Stack                   |
 *              +-------------+--------------------------------+
 * */

#define P_LIMIT             0x3F000000
#define P_RESERVED_TOP      0x01800000
#define P_TIMESTACK_TOP     0x01800000 // update value in `asm.S`
#define P_ENVS_BASE         0x00B00000
#define P_PAGES_BASE        0x00500000
#define P_PGDIR_BASE        0x00200000
#define P_STACK_TOP         0x00080000

#define K_TIMESTACK_TOP     KADDR(P_TIMESTACK_TOP)
#define K_ENVS_BASE         KADDR(P_ENVS_BASE)
#define K_PAGES_BASE        KADDR(P_PAGES_BASE)
#define K_PGDIR_BASE        KADDR(P_PGDIR_BASE)
#define K_STACK_TOP         KADDR(P_STACK_TOP)

#define U_PAGES_BASE        0x90000000
#define U_ENVS_BASE         0x80000000
#define U_LIMIT             0x80000000
#define U_XSTACK_TOP        0x80000000
#define U_STACK_TOP         0x01000000


#define UVPD                0x7040200000UL
#define UVPM                0x7040000000UL
#define UVPT                0x7000000000UL


#define PTE_NORMAL      (0 << 2)
#define PTE_DEVICE      (1 << 2)
#define PTE_NON_CACHE   (2 << 2)

#define PTE_4KB         (0b11)
#define PTE_KERN        (0 << 6)
#define PTE_USER        (1 << 6)
#define PTE_RW          (0 << 7)
#define PTE_RO          (1 << 7)
#define PTE_AF          (1 << 10)
#define PTE_PXN         (1UL << 53)
#define PTE_UXN         (1UL << 54)
#define PTE_OUTER_SHARE (2 << 8)
#define PTE_INNER_SHARE (3 << 8)

// user-defined PTE_FLAG
#define PTE_COW         (1UL << 55)
#define PTE_LIBRARY     (1UL << 56)


#endif //OSLABPI_MMU_H
