# host software for adc
runs on the host pc with  all the comforts of `std`, receives ADC measurements from the teensy over /dev/ttyACM0
if needed we can stream the frames as bytes, for now they are base64 encoded in logs prefixed with `db64`

