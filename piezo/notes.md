# Piezo Preamp SPICE Simulation Notes

## Overview
Created a SPICE simulation environment for designing a piezo microphone preamplifier with output suitable for ADC input (0-3.3V range).

## Setup
1. Created `shell.nix` with ngspice, python3, matplotlib, numpy, and gnuplot
2. Enter environment with: `nix-shell`

## Circuit Files
- `piezo_preamp.cir` - Initial attempt (had convergence issues)
- `piezo_preamp_fixed.cir` - Working circuit with:
  - Piezo model: 1nF capacitance, 10MΩ impedance, 50mV signal
  - Input buffer with high impedance
  - DC bias at VDD/2 (1.65V)
  - AC coupling between stages
  - Gain stage (10x amplification)
  - Output clamping diodes

## Running Simulations
```bash
ngspice -b piezo_preamp_fixed.cir -o piezo_preamp_fixed.log
```

## Plotting Results
Created `plot_results.gp` gnuplot script that generates:
- `transient_response.png` - Full transient response
- `transient_zoom.png` - First 1ms detail
- `output_voltage.png` - Output with ADC limits marked

Run with: `gnuplot plot_results.gp`

## Issues Found
The output is swinging negative beyond ADC range (below 0V). Circuit needs:
- Better DC biasing
- Proper limiting/clamping
- Possibly single-supply op-amp design adjustments

## Next Steps
- Fix negative swing issue
- Add proper rail-to-rail output stage
- Consider using real op-amp models instead of ideal sources
- Optimize gain for typical piezo signal levels