pub const DEFAULT_SVC_ORG: usize = 1024 * 1024 * 2 / 4 - 512; // 2 MB - 2 KB
pub const DEFAULT_OS: &str = "; Default Interrupt handlers

ExceptionHandler0  hcf ; Overflow
ExceptionHandler1  hcf ; Zero div
ExceptionHandler2  hcf ; Unknow instruction
ExceptionHandler3  hcf ; Memory protection
ExceptionHandler4  hcf ; unused


InterruptHandler5  hcf ; Memory parity error
InterruptHandler6  hcf ; Timer interrupt
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


; This takes way too long
SVC15       pushr   sp          ; SVC DATE
            in      r1, =rtc    ;
            load    r2, r1      ; 
            load    r3, =3600   ;
            mul     r3, =24     ;
            mod     r2, r3      ; R2 is now hr/min/sec part of timestamp
            sub     r1, r2      ; R1 is now timestamp without hr/min/sec
            div     r1, r3      ; R1 is now days since epoch
            load    r2, =1970   ; R2 is current year
            loop nop
                push    sp, =0          ;
                push    sp, r2          ;
                call    sp, DaysInYear  ;
                pop     sp, r5          ; Number of days in this year
                comp    r1, r5          ; If R1 contains less days than this year, exit loop
                jles    EndYear         ;
                sub r1, r5              ; Remove this years days from R1
                add r2, =1              ; Add 1 to current year
                jump loop    
            EndYear store r2, @-4(fp)   ;

                                        ; R2 = Current Month
            load    r2,     =1          ; January
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            add     r2,     =1          ; February
            load    r3,     r5          ; load days in this year. 
            sub     r3,     =365        ; R3 can be 0 or 1, depending on leap year.
            add     r3,     =28         ; Make it 28 or 29
            comp    r1,     r3          ; 
            jles    EndMon              ;
            sub     r1,     r3          ;

            add     r2,     =1          ; March
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            add     r2,     =1          ; April
            comp    r1,     =30         ;
            jles    EndMon              ;
            sub     r1,     =30         ;

            add     r2,     =1          ; May
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            add     r2,     =1          ; June
            comp    r1,     =30         ;
            jles    EndMon              ;
            sub     r1,     =30         ;

            add     r2,     =1          ; July
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            add     r2,     =1          ; August
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            add     r2,     =1          ; September
            comp    r1,     =30         ;
            jles    EndMon              ;
            sub     r1,     =30         ;

            add     r2,     =1          ; October
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            add     r2,     =1          ; November
            comp    r1,     =30         ;
            jles    EndMon              ;
            sub     r1,     =30         ;

            add     r2,     =1          ; December
            comp    r1,     =31         ;
            jles    EndMon              ;
            sub     r1,     =31         ;

            EndMon store r2, @-3(fp)    ;
            add r1, =1                  ;
            EndDay store r1, @-2(fp)    ;
            iexit   sp, =3              ;
            

                ; -3(fp) return value: 365 or 366.
                ; -2(fp) arg0: year
    DaysInYear  pushr   sp                  ;
                load    r4, =365            ; Default case: Not leap year
                load    r5, -2(fp)          ;
                mod     r5, =4              ; 
                jnzer   r5, DaysInYearEnd   ; Not divisible by 4: Exit, aint a leap year
                load    r4, =366            ; 
                load    r5, -2(fp)          ;
                mod     r5, =100            ; 
                jnzer   r5, DaysInYearEnd   ; Not divisible by 100: Exit, is a leap year
                load    r4, =365            ; 
                load    r5, -2(fp)          ;
                mod     r5, =400            ; 
                jnzer   r5, DaysInYearEnd   ; Not divisible by 400: Exit, aint a leap year
                load    r4, =366            ; 

        DaysInYearEnd nop                   ;
                store   r4, -3(fp)          ;
                popr    sp                  ;
                exit    sp, =1              ;
";
