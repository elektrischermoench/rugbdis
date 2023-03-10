# rugbdis
A rust based game boy disassembler

## Experimental

This is for learning rust and doing some cool stuff with disassembling and Game Boy (TM).

## Example

Building an Hello World with https://github.com/gbdk-2020/gbdk-2020

```C
#include <stdio.h>
 
void main()
{
int counter = 1;
 
while (counter <=16)
    {
    printf("\nHello World!");
    counter++;
    }
}
```

Build with:

```bash
./gbdk/bin/lcc -o helloworld.gb helloworld.c
```

Executed with https://sameboy.github.io

![image](https://user-images.githubusercontent.com/5123349/212653165-1b52b60f-1c81-4e80-b01f-0dbfcf48a19b.png)

Unfortunately the compiler seem to break the entry point. This is working with other roms.

```bash
Title: ^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@^@
Cartridge type: ROM ONLY
Destination code (Japanese Version): true
Super GameBoy: true
Color GameBoy: true
RAM size: None
ROM size: 32kB
Global checksum: AE3
Header checksum: 53
Entrypoint: 0xFFFF
100:     18 55          JR 0x55
102:     ff             RST 38H
103:     ff             RST 38H
104:     ce ed          ADC A 0xed
106:     66             LD H HL
107:     66             LD H HL
```

Compared with r2 this disassembler seems to work.

```bash
% r2 helloworld.gb
 -- Don't wait for Travis
[0x00000100]> pd 10
            ;-- entry0:
            ;-- pc:
        ,=< 0x00000100      1855           jr 0x55
        |   0x00000102      ff             rst sym.rst_56
        |   0x00000103      ff             rst sym.rst_56
        |   0x00000104      ceed           adc 0xed
        |   0x00000106      66             ld h, [hl]
        |   0x00000107      66             ld h, [hl]
        |   0x00000108      cc0d00         call Z, 0x000d
        |   0x0000010b      0b             dec bc
        |   0x0000010c      03             inc bc
        |   0x0000010d      73             ld [hl], e
[0x00000100]> 
```

If used with dumbed ROMS the entrypoint is parsed correctly (only use ROMS you own physically):

```
...
Destination code (Japanese Version): false
Super GameBoy: false
Color GameBoy: false
RAM size: None
ROM size: 128kB
Global checksum: 8DC4
Header checksum: 4C
Entrypoint: 0x164
100:     0              NOP
101:     c3 64 1        JP 0x164
...
```
