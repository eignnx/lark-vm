#include "../lark.customasm"
#bank rom

j	_start

boot:
	; Setup ILL_INSTR handler:
	li	$t0, -2 ; Slot in interrupt vector
	li	$t1, handle__ILL_INSTR`15 ; Function-pointer of handler
	sw	0($t0), $t1
	jr	$ra

; <FnDef name=_start args=[]>
_start:
	jal	$ra, boot

; <Section id="print-test-exn">
	li	$a0, strlit__QQTest_exnQQ__0
	li	$a1, 19
	exn	3
; </Section>

	exn	0 ; Illegal instruction!

	halt	
; </FnDef>

; Illegal instruction handler
handle__ILL_INSTR:
	; <Preserve>
	mv	$k0, $a0
	mv	$k1, $a1
	; </Preserve>

	li	$a0, strlit__QQInside_handlerQQ__0
	li	$a1, 15
	exn	3

	; <Restore>
	mv	$a0, $k0
	mv	$a1, $k1
	; </Restore>

	kret

strlit__QQTest_exnQQ__0:
	#d "Test exn DEBUG_PUTS"

strlit__QQInside_handlerQQ__0:
	#d "Inside handler!"
