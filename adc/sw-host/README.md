# host software for adc
runs on the host pc with  all the comforts of `std`, receives ADC measurements from the teensy over /dev/ttyACM0
if needed we can stream the frames as bytes, for now they are base64 encoded in logs prefixed with `db64`

## build
`cargo build` to build
`cargo run` to run

## goals
1. simple immediate-mode gui runs on aarch64 linux but ideally elsewhere too
receives base64 encoded and later binary encoded packets over /dev/ttyACM0
see `struct Frame` for structure of data

2. displays live measurements on the screen as an 8 channel line plot
you should probably keep stuff in a vecdeque for this

3. recording to disk triggerable from keyboard press or gui mouse click.
format should be something very simple, maybe a .wav file? if those can have 8 channels. if not just anything numpy can read like raw values

