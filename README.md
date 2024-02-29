A third-party tool to load and execute binaries over UART for Mediatek SoCs.

The BootROM recovery support on Mediatek smartphone SoCs is also available on their ARM-based router SoCs over UART. However, there's no SP Flash Tool and accompanying download agent released for these router chips.

This tool implements basic support to load and execute a custom binary over UART as the download agent on Mediatek ARM SoCs with secure-boot disabled. With this, it's possible to recover Mediatek ARM routers over UART even if the bootloader in flash is completely broken.

# Credits
The UART recovery protocol is exactly the same as the USB one on MTK smartphone SoCs, which has already been reverse-engineered by others. The knowledge of the protocol details come from these two projects:

[bkerler/mtkclient](https://github.com/bkerler/mtkclient)

[MTK-bypass/bypass_utility](https://github.com/MTK-bypass/bypass_utility)

# Compatibility

This utility should work on all Mediatek SoCs with secure-boot disabled. It's been tested on MT7622/MT7629 and MT798x.

This utility won't work on secure-boot enabled routers.

# Usage

```
Usage: mtk_uartboot [OPTIONS] --payload <PAYLOAD>

Options:
  -s, --serial <SERIAL>
          Serial port
  -p, --payload <PAYLOAD>
          Path to the binary code to be executed
  -l, --load-addr <LOAD_ADDR>
          Load address of the payload [default: 2101248]
  -a, --aarch64
          Whether this is an aarch64 payload
  -f, --fip <FIP>
          Path to an FIP payload. Use this to start an FIP using MTK BL2 built with UART download support
      --brom-load-baudrate <BROM_LOAD_BAUDRATE>
          Baud rate for loading bootrom payload [default: 460800]
      --bl2-load-baudrate <BL2_LOAD_BAUDRATE>
          Baud rate for loading bl2 payload [default: 921600]
  -h, --help
          Print help
```

Load and start a bootloader on ARM64 SoCs:

```
./mtk_uartboot -s /dev/ttyUSB0 -p da.bin --aarch64
```

Omit --aarch64 if you are working with ARMv7 SoCs.

If there's only one serial interface available on your system, you can omit -s as well. The program will use the first serial port it finds.

This utility also supports a UART-boot protocol available in BL2 on Mediatek routers. When using such a BL2 built with UART recovery as download agent, it can subsequently load and start an FIP after BL2 is started:

```
./mtk_uartboot -p bl2.bin --aarch64 -f bl31-uboot.fip
```
