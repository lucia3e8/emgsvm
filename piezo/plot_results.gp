# Gnuplot script for piezo preamp simulation results

# Set output to PNG files
set terminal png size 1200,800 font "Arial,12"

# Plot 1: Transient Response - Input and Output voltages
set output "transient_response.png"
set title "Piezo Preamp Transient Response"
set xlabel "Time (seconds)"
set ylabel "Voltage (V)"
set grid
set key top right

# Column mapping from piezo_preamp_tran.txt:
# Column 2: time
# Column 4: v(in) 
# Column 6: v(out)
# Column 8: v(vbias)

plot "piezo_preamp_tran.txt" using 2:4 with lines lw 2 title "Input (piezo)", \
     "piezo_preamp_tran.txt" using 2:6 with lines lw 2 title "Output", \
     "piezo_preamp_tran.txt" using 2:8 with lines lw 1 dt 2 title "Vbias (1.65V)"

# Plot 2: Zoomed transient response (first 1ms)
set output "transient_zoom.png"
set title "Piezo Preamp Transient Response (First 1ms)"
set xrange [0:0.001]
set yrange [0:3.3]
replot

# Plot 3: Output voltage only with limits
set output "output_voltage.png"
set title "Preamp Output Voltage"
set xlabel "Time (seconds)"
set ylabel "Output Voltage (V)"
set xrange [*:*]
set yrange [-0.5:3.8]
set arrow from graph 0,first 0 to graph 1,first 0 nohead lc rgb "red" lw 2
set arrow from graph 0,first 3.3 to graph 1,first 3.3 nohead lc rgb "red" lw 2
set label "ADC Min (0V)" at graph 0.02,first 0.2 tc rgb "red"
set label "ADC Max (3.3V)" at graph 0.02,first 3.1 tc rgb "red"

plot "piezo_preamp_tran.txt" using 2:6 with lines lw 2 lc rgb "blue" title "Output"

print "Plots generated: transient_response.png, transient_zoom.png, output_voltage.png"