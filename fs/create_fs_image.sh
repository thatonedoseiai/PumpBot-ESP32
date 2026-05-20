#!/bin/sh
# we need to install littlefs-python before we can use this.
# Create a venv and install littlefs-python through pip. 
# Finally, use espflash to flash this onto the correct partition.

./littlefs-python create contents pb_fs.bin --fs-size=2mb --block-size=4096
espflash write-bin 0x210000 pb_fs.bin