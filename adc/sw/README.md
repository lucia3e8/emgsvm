# adc to spi to computer
iN rUsT to keep the toolchain complicated


## pins
ADS131M08 -> Teensy4.1
SYNC -> 2
DRDY -> 9
CS -> 10 (CS)       | LPSPI4
DIN -> 11 (MOSI)    |
DOUT -> 12 (MISO)   |
SCLK -> 13 (SCK)    |
CLKIN -> 23

I made a silly mistake in PCB design and pin 13 is connected to 27
You have to leave 27 floating

## build and flash

```
nix-shell
cargo objcopy --verbose --release -- -O ihex simsamadc.hex
sudo teensy-loader-cli --mcu TEENSY41 -w simsamadc.hex
```
then press button on teensy
