# adc to spi to computer
iN rUsT to keep the toolchain complicated


## pins
ADS131M08 -> Teensy4.1
SYNC -> 2
DRDY -> 9
CS -> 10 (CS)
DIN -> 11 (MOSI)
DOUT -> 12 (MISO)
SCLK -> 27 (SCK1)
CLKIN -> 23

## build and flash

```
nix-shell
cargo objcopy --verbose --release -- -O ihex simsamadc.hex
sudo teensy-loader-cli --mcu TEENSY41 -w simsamadc.hex
```
then press button on teensy
