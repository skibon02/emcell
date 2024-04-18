MEMORY
{
  CELL1_FLASH : ORIGIN = 0x08000000, LENGTH = 510K
  FLASH : ORIGIN = 0x08080000, LENGTH = 510K

  /* ABI */
  CELL1_ABI : ORIGIN = 0x0807FC00, LENGTH = 1K
  CELL2_ABI : ORIGIN = 0x080FFC00, LENGTH = 1K

  /* RAM */
  /* <--- leave 24K for the stack */
  CELL1_RAM : ORIGIN = 0x20006000, LENGTH = 32K
  RAM : ORIGIN = 0x2000e000, LENGTH = 40K
}

_stack_start = ORIGIN(CELL1_RAM);

SECTIONS {
    .CELL2_ABI ORIGIN(CELL2_ABI) : {
        . = ALIGN(4);
        KEEP(*(.emcell.CELL2ABI));
        . = ALIGN(4);
    } > CELL2_ABI

    .CELL1_ABI ORIGIN(CELL1_ABI): {
        _emcell_Cell1ABI_internal = .;
    } > CELL1_ABI
}