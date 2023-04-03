pub const DEFAULT_SVC_ORG: usize = 0x2000 - 512; // 32KB - 2 KB
pub const DEFAULT_OS: &str = "
; Default Interrupt handlers
ExceptionHandler0  hcf ; Overflow
ExceptionHandler1  hcf ; Zero div
ExceptionHandler2  hcf ; Unknow instruction
ExceptionHandler3  hcf ; Memory protection
ExceptionHandler4  hcf ; unused


InterruptHandler5  hcf ; Memory parity error

; Timer interrupt
;     The handler will just clear the interrupt and return.
;     This is till useful because the machine wakes up from halt.
InterruptHandler6       pushr sp        ;
                        load  r0, =0    ; load 0
                        out   r0, =0x20 ; send it to PIC command
                        popr  sp        ;
                        iexit sp, =0    ;

InterruptHandler7  hcf ; kbd
InterruptHandler8  hcf ; mouse
InterruptHandler9  hcf ; disc drive
InterruptHandler10  hcf ; printer

SVC11       hcf                 ; SVC HALT; Halt permanently

SVC12       pushr   sp          ; SVC READ
            in      r1, =kbd    ;
            load    r2, @-9(sp) ;
            store   r1, @r2     ;
            popr    sp          ;
            iexit   sp, =1;     ;

SVC13       pushr   sp          ; SVC WRITE
            load    r1, -9(sp)  ;
            out     r1, =crt    ;
            popr    sp          ;
            iexit   sp, =1;     ;

SVC14       pushr   sp          ; SVC TIME
            in      r1, =rtc    ;
            load    r2, r1      ; Sec
            mod     r2, =60     ;
            store   r2, @-9(sp) ;
            div     r1, =60     ;
            load    r2, r1      ; Min
            mod     r2, =60     ;
            store   r2, @-10(sp);
            div     r1, =60     ;
            load    r2, r1      ; Hr
            mod     r2, =24     ;
            store   r2, @-11(sp);
            popr    sp,         ;
            iexit   sp, =3      ;


SVC15       pushr   sp          ; Date
            in      r1, =rtc    ; Get Unix time 
            load    r2, r1      ; 
            load    r3, =3600   ; R3 will contain number of seconds in a day.
            mul     r3, =24     ; 
            mod     r2, r3      ; R2 now contains time of day in seconds.
            sub     r1, r2      ; R1 now contais date in number of seconds, without time of day.
            div     r1, r3      ; R1 now contains date in number of days.
            load    r2, =1970   ; R2 will contain current year.
            load    r3, r1      ; 
            mod     r3, =1461   ; Number of days in current 4-year cycle since epoch.
            sub     r1, r3      ; 
            div     r1, =1461   ; Now R1 contains number of 4-year cycles since epoch.
            mul     r1, =4      ; 
            add     r2, r1      ; Now R2 contains the start of current 4y cycle 
            
            load    r1, =0      ; Leap day

            comp    r3, =365    ; 1st year in cycle
            jles    yr_done     ;
            sub     r3, =365    ; Subtract this years days from days left
            add     r2, =1      ; 1 to current year

            comp    r3, =365    ; 2nd year in cycle
            jles    yr_done     ;
            sub     r3, =365    ;
            add     r2, =1      ; 

            load    r1, =1      ; leap day
            comp    r3, =365    ; 
            jles    yr_done     ; 3rd year in cycle
            load    r1, =0      ; Leap day
            sub     r3, =366    ;
            add     r2, =1      ;

yr_done    store r2, @-12(sp)           ; Store Year

            load    r2,     =1          ; January
            comp    r3,     =31         ;
            jles    mo_done             ;
            sub     r3,     =31         ;

            add     r2,     =1          ; February
            comp    r3,     =28(R1)     ; R1 adds possible leap day
            jles    mo_done             ;
            sub     r3,     =28(R1)     ;

            add     r2,     =1          ; March
            comp    r3,     =31         ;
            jles    mo_done             ;
            sub     r3,     =31         ;

            add     r2,     =1          ; April
            comp    r3,     =30         ;
            jles    mo_done             ;
            sub     r3,     =30         ;

            add     r2,     =1          ; May
            comp    r3,     =31         ;
            jles    mo_done             ;
            sub     r3,     =31         ;

            add     r2,     =1          ; June
            comp    r3,     =30         ;
            jles    mo_done             ;
            sub     r3,     =30         ;

            add     r2,     =1          ; July
            comp    r3,     =31         ;
            jles    mo_done             ;
            sub     r3,     =31         ;

            add     r2,     =1          ; August
            comp    r3,     =31         ;
            jles    mo_done             ;
            sub     r3,     =31         ;

            add     r2,     =1          ; September
            comp    r3,     =30         ;
            jles    mo_done             ;
            sub     r3,     =30         ;

            add     r2,     =1          ; October
            comp    r3,     =31         ;
            jles    mo_done             ;
            sub     r3,     =31         ;

            add     r2,     =1          ; November
            comp    r3,     =30         ;
            jles    mo_done             ;
            sub     r3,     =30         ;

            add     r2,     =1          ; December

    mo_done out     r2,     =crt        ; print month
            store   r2, @-11(sp)        ; Store month
            add     r3,     =1          ; Calendar days don't start from zero.
            store   r3, @-10(sp)        ; Store day
            popr    sp                  ;
            iexit   sp, =3              ;
";
