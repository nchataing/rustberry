Rustberry bootloader
====================

Motivation
----------
If you have not tested, it is REALLY boring to change the kernel.img file by
hand every time when developping.
Debugging on hardware is already hard and you often want to tweak your code by
few lines of code a lot of time.
That's why having an UART bootloader is more than useful.

All the bootloaders we found so far were buggy on Raspberry Pi 2 hardware or
not supporting the `kernel_old=1` mode.
Therefore we have made our own, but reusing the existing Raspbootin to be able
to use its Raspbootcom client.

How to use it
-------------
First compile the bootloader with `make bootloader` at the project root.

Flash the file `bootloader.img` as `kernel.img` on the usual `boot` partition
of the SD card.

Add a file `config.txt` with at least the following options:
```
kernel_old=1
boot_delay=3
```

The boot delay can be tweaked a bit, but on my computer Linux fails to
recognize the UART soon enough if using the default configuration.

Then you can use Raspbootcom to send a kernel to the Raspberry Pi.
There are some restrictions on this kernel :
- It will be booted as if the `kernel_old=1` option was set.
  This mean that the execution begins with the 4 cores starting at 0x0.
- It will skip adresses 0x0100 to 0x8000 during copy and fail if they were not
  set to zero in the input image. See the next section for the reasons.

How it works
------------
The bootloader is itself a standard bare-metal application.
It begins its execution at 0x0 but quickly separate the cores and move all the
program counters toward a safe zone around 0x4000.
This must be done in order to enable the bootloader to overwrite the adresses
between 0x0 and 0x100 with the real kernel.

The cores 1, 2 & 3, are trapped in a loop waiting for the reset signal (a write
at 0x2000) from the core 0. Then they wipe their cache and reset their
program counter to 0x0.

The core 0 executes the Raspbootin protocol but start at address 0x0 and
carefully skip adresses from 0x100 to 0x8000 to avoid wiping away the ATAGs
and its own code and stack.
To prevent accidental deletion of useful data in this zone, it aborts if a non
zero value is found there.

When ready to boot (i.e. the kernel has been entirely recieved), it notifies
the cores 1, 2 & 3 to continue their execution and resets.

