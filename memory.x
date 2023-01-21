SECTIONS {
    . = DEFINED(ALGO_PLACEMENT_START_ADDRESS) ? ALGO_PLACEMENT_START_ADDRESS : 0x0;

    /*
     * The PrgCode output section name comes from the CMSIS-Pack flash algorithms
     * templates and armlink. It is used here because several tools that work
     * with these flash algorithms expect this section name.
     *
     * All input sections are combined into PrgCode because RWPI using R9 is not
     * currently stable in Rust, thus having separate PrgData sections that the
     * debug host might locate at a different offset from PrgCode is not safe.
     */
    PrgCode : {
        KEEP(*(.entry))
        KEEP(*(.entry.*))

        *(.text)
        *(.text.*)

        *(.rodata)
        *(.rodata.*)

        *(.data)
        *(.data.*)

        *(.sdata)
        *(.sdata.*)
        
        *(.bss)
        *(.bss.*)

        *(.uninit)
        *(.uninit.*)

        . = ALIGN(4);
    }

    /DISCARD/ : {
        /* Unused exception related info that only wastes space */
        *(.ARM.exidx);
        *(.ARM.exidx.*);
        *(.ARM.extab.*);
    }
}
