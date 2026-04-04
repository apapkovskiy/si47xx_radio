MEMORY
{
  /* NOTE 1 K = 1 KiBi = 1024 bytes */
  /* These values correspond to the NRF5340 */
  FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
  RAM : ORIGIN = 0x20000000, LENGTH = 256K
}

SECTIONS
{
  linkme :{
    __start_linkme_OPTIONS = .;
    *(linkme_OPTIONS*);
    __stop_linkme_OPTIONS = .;
  } > FLASH
} INSERT AFTER .text;
