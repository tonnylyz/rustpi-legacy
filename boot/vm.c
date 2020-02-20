#include "mmu.h"

typedef unsigned long usize_t;

usize_t freemem __attribute__ ((section (".data.vm_init")));

usize_t *boot_alloc() __attribute__ ((section (".text.vm_init")));
usize_t *boot_alloc() {
    usize_t *alloced_mem;
    usize_t i;
    alloced_mem = (usize_t *)freemem;
    freemem += BY2PG;
    i = 0;
    while (i < BY2PG / sizeof(usize_t)) {
        *(alloced_mem + i) = 0;
        i ++;
    }
    return alloced_mem;
}

usize_t *boot_pgdir_walk(usize_t *pgdir, usize_t va) __attribute__ ((section (".text.vm_init")));
usize_t *boot_pgdir_walk(usize_t *pgdir, usize_t va) {
    // Use 39bit virtual address (4kB page and 3 levels page table)
    // [   9   |   9   |   9   |   12   ]
    usize_t *pde;
    usize_t *pme;
    usize_t *pte;
    pde = (usize_t *) (&pgdir[PDX(va)]);
    pme = (usize_t *) (PTE_ADDR(*pde)) + PMX(va);
    if (!(*pde & PTE_4KB)) {
        pme = boot_alloc();
        *pde = (usize_t) pme | PTE_KERN | PTE_RW | PTE_AF | PTE_4KB;
        pme += PMX(va);
    }
    pte = (usize_t *) (PTE_ADDR(*pme)) + PTX(va);
    if (!(*pme & PTE_4KB)) {
        pte = boot_alloc();
        *pme = (usize_t) pte | PTE_KERN | PTE_RW | PTE_AF | PTE_4KB;
        pte += PTX(va);
    }
    return pte;
}

void boot_map_segment(usize_t *pgdir, usize_t va, usize_t size, usize_t pa, int perm) __attribute__ ((section (".text.vm_init")));
void boot_map_segment(usize_t *pgdir, usize_t va, usize_t size, usize_t pa, int perm) {
    usize_t *pte;
    int i;
    for (i = 0; i < size; i += BY2PG) {
        pte = boot_pgdir_walk(pgdir, va + i);
        *pte = PTE_ADDR(pa + i) | perm | PTE_KERN | PTE_RW | PTE_AF | PTE_4KB;
    }
}

void vm_init() __attribute__ ((section (".text.vm_init")));
void vm_init() {
    usize_t *pgdir;
    freemem = P_PGDIR_BASE;
    pgdir = boot_alloc();
    // 0x00000000 - 0x3F000000 Normal memory
    boot_map_segment(pgdir, 0, P_LIMIT, 0, PTE_NORMAL | PTE_INNER_SHARE);
    // 0x3F000000 - 0x40000000 Device I/O
    boot_map_segment(pgdir, P_LIMIT, 0x01000000, P_LIMIT, PTE_DEVICE | PTE_OUTER_SHARE);
    // One more page for control Regs
    boot_map_segment(pgdir, 0x40000000, BY2PG, 0x40000000, PTE_DEVICE | PTE_OUTER_SHARE);
}

