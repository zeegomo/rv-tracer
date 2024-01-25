/* 1K = 1 KiBi = 1024 bytes */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM : ORIGIN = 0x20000000, LENGTH = 64K
}

ENTRY(main);

SECTIONS
{
  .text :
  {
    *(.text .text.*);
  } > RAM
}
