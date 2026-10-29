; ============================================================================
;  KC.ASM - the loader for Kirin Commander on DOS.
;
;      FASM KC.ASM KC.COM
;
;  KC.COM puts KC.BIN - the commander, built from src/main.rs - in extended
;  memory under DPMI and jumps into it. The loading is OWLOSUI's own
;  lib/dos/LOADER.INC: four megabytes, two selectors, a jump.
; ============================================================================

LOG = 0
        include '..\..\OWLOSUI\lib\dos\LOADER.INC'
