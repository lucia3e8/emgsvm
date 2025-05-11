import pyvisa as visa
import numpy as np
import matplotlib.pyplot as plt
import time
from datetime import datetime

def succeed(scope, command):
    """Execute a command and throw if there's an error, with human-readable ESR decoding."""
    scope.write(command)
    esr = int(scope.query("*ESR?"))
    if esr != 0:
        errors = []
        if esr & (1 << 7): errors.append("Power On")
        if esr & (1 << 6): errors.append("User Request")
        if esr & (1 << 5): errors.append("Command Error (Invalid command or parameter)")
        if esr & (1 << 4): errors.append("Execution Error (Valid command but couldn't execute)")
        if esr & (1 << 3): errors.append("Device Dependent Error")
        if esr & (1 << 2): errors.append("Query Error (Data not available/lost)")
        if esr & (1 << 1): errors.append("Request Control")
        if esr & (1 << 0): errors.append("Operation Complete")
        
        error_details = scope.query(":SYSTem:ERRor?").strip()
        raise RuntimeError(f"Command '{command}' failed with ESR {esr}:\n" + 
                         "\n".join(errors) + f"\nScope error: {error_details}")

# Example usage:
# succeed(scope, "*RST")
# succeed(scope, ":WAVeform:DATA?")

rm = visa.ResourceManager()
print("Available resources:")
print(rm.list_resources())
scope = rm.open_resource('USB0::0xF4EC::0x1011::SDS2PCBX4R0394::INSTR') 
scope.timeout = 10000  # ms


# Configure the oscilloscope for your signal
channel = 1  # Using channel 1
succeed(scope, "*RST")  # Reset the scope
time.sleep(0.5)

# Configure for your 4V-7.3V signal
succeed(scope, f":CHANnel{channel}:SCALe 0.5")  # Set voltage scale to 1V/div
succeed(scope, f":CHANnel{channel}:OFFSet 0.0")  # Center around 5.5V (midpoint of 4-7.3V)
# succeed(scope, f":CHANnel{channel}:DISPlay ON")  ESR 32
# Set probe attenuation to 10X for channel 1
# TODO: figure out how to set probe attenuation to 10X
# this command does not work
# scope.write(f":CHANnel{channel}:PROBe 10")  # 10X probe attenuation

# Set time scale - adjust based on how much detail you want
# For continuous data, we'll set a reasonable time base
succeed(scope, ":TIMebase:SCALe 0.01")  # 0.5 sec/div = 5 seconds across 10 divisions

# Set to auto trigger mode to continuously capture regardless of trigger conditions
succeed(scope, ":TRIGger:MODE AUTO")

# Set up acquisition parameters
# succeed(scope, "ACQuire:MDEPth 140000") ESR 16
succeed(scope, ":WAVeform:POINts:MODE RAW")
succeed(scope, ":WAVeform:POINts 140000")  # Request maximum points
succeed(scope, ":WAVeform:FORMat BYTE")
succeed(scope, f":WAVeform:SOURce CHANnel{channel}")

# Create a directory for saving multiple captures
import os
timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
save_dir = f"scope_capture_{timestamp}"
os.makedirs(save_dir, exist_ok=True)

# Function to capture and save waveform
def capture_and_save(capture_number):
    # Run for a moment to collect new data
    succeed(scope, ":RUN")
    time.sleep(0.1)  # Short acquisition time
    succeed(scope, ":STOP")
    
    # Read back actual settings (for scaling)
    time_scale = float(scope.query(":TIMebase:SCALe?"))
    vertical_scale = float(scope.query(f":CHANnel{channel}:SCALe?"))
    vertical_offset = float(scope.query(f":CHANnel{channel}:OFFSet?"))
    
    # Capture waveform data
    succeed(scope, ":WAVeform:DATA?")
    data = scope.read_raw()
    
    # Parse the header correctly
    # Format is "DAT2,#9010000000" where:
    # - "DAT2," is the prefix
    # - "#" indicates start of length encoding
    # - "9" indicates that next 9 digits specify the data length
    # - "010000000" is the actual data length
    header_offset = data.find(b'#') + 2  # Skip the '#' and the digit that follows
    data_size = int(data[header_offset:header_offset+9])
    data_offset = header_offset + 9
    data = data[data_offset:data_offset+data_size]  # Extract just the waveform data
    
    # Convert binary data to numerical values
    wave_data = np.frombuffer(data, dtype=np.uint8)
    
    # Apply scaling to convert to voltage
    vertical_scale_factor = vertical_scale / 25  # Convert divisions to actual values
    voltage_data = (wave_data - 128) * vertical_scale_factor - vertical_offset
    
    # Create time axis
    time_data = np.linspace(0, time_scale * 10, len(voltage_data))  # 10 divisions across
    
    # Save data to CSV
    filename = f"{save_dir}/waveform_{capture_number}.csv"
    np.savetxt(filename, np.column_stack((time_data, voltage_data)), delimiter=",", 
               header="Time (s),Voltage (V)", comments="")
    
    return len(voltage_data)  # Return number of samples captured

# Capture data continuously for 5 seconds
start_time = time.time()
capture_count = 0
total_samples = 0

print(f"Starting 5-second continuous capture...")
while time.time() - start_time < 5:  # Run for 5 seconds
    capture_count += 1
    samples = capture_and_save(capture_count)
    total_samples += samples
    print(f"Capture #{capture_count}: {samples} samples")

end_time = time.time()
duration = end_time - start_time

print(f"\nCapture complete!")
print(f"Captured {capture_count} waveforms in {duration:.2f} seconds")
print(f"Total samples: {total_samples}")
print(f"Effective sample rate: {total_samples/duration:.2f} samples/second")
print(f"Data saved to {save_dir}/ directory")

# Close connection
scope.close()

# Optional: Plot the last capture
last_data = np.loadtxt(f"{save_dir}/waveform_{capture_count}.csv", delimiter=',', skiprows=1)
plt.figure(figsize=(10, 6))
plt.plot(last_data[:, 0], last_data[:, 1])
plt.xlabel("Time (s)")
plt.ylabel("Voltage (V)")
plt.title(f"Waveform Capture #{capture_count}")
plt.grid(True)
plt.savefig(f"{save_dir}/final_waveform.png")
plt.show()