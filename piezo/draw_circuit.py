#!/usr/bin/env python3
import sys

def generate_graphviz():
    """Generate a Graphviz DOT file for the circuit"""
    
    dot_content = """digraph PiezoPreamp {
    rankdir=LR;
    graph [fontname="Arial", splines=ortho];
    node [shape=box, style=filled, fillcolor=lightblue, fontname="Arial"];
    edge [fontname="Arial"];
    
    // Define nodes
    subgraph cluster_input {
        label="Input Stage";
        style=dotted;
        Vpiezo [label="Piezo\\n50mV AC", shape=circle, fillcolor=lightgreen];
        Cpiezo [label="Cpiezo\\n1nF", shape=box, fillcolor=lightyellow];
        Rpiezo [label="Rpiezo\\n10MΩ", shape=box, fillcolor=lightyellow];
    }
    
    subgraph cluster_buffer {
        label="Unity Gain Buffer";
        style=dotted;
        Buffer1 [label="Buffer\\nG=1\\nHigh-Z", shape=triangle, fillcolor=lightcoral];
    }
    
    subgraph cluster_coupling {
        label="AC Coupling";
        style=dotted;
        C_ac [label="C_ac\\n1µF", shape=box, fillcolor=lightyellow];
        R_dc [label="R_dc\\n1MΩ", shape=box, fillcolor=lightyellow];
    }
    
    subgraph cluster_amp {
        label="Gain Stage";
        style=dotted;
        Amp2 [label="Amp\\nG=10", shape=triangle, fillcolor=lightcoral];
        Rf [label="Rf=9k", shape=box, fillcolor=lightyellow];
        Rg [label="Rg=1k", shape=box, fillcolor=lightyellow];
    }
    
    subgraph cluster_output {
        label="Output Stage";
        style=dotted;
        Clamp [label="Clamp\\nDiodes", shape=diamond, fillcolor=orange];
        R_load [label="R_load\\n100kΩ", shape=box, fillcolor=lightyellow];
        Output [label="Output\\n0-3.3V", shape=circle, fillcolor=lightgreen];
    }
    
    // Power and bias
    VDD [label="VDD\\n3.3V", shape=invhouse, fillcolor=cyan];
    VSS [label="VSS\\n0V", shape=invhouse, fillcolor=gray];
    Vbias [label="Vbias\\n1.65V\\n(R divider)", shape=hexagon, fillcolor=lightgreen];
    
    // Connections
    Vpiezo -> Cpiezo;
    Cpiezo -> Rpiezo;
    Rpiezo -> Buffer1 [label="High-Z input"];
    Buffer1 -> C_ac;
    C_ac -> Amp2;
    C_ac -> R_dc [style=dashed, label="DC path"];
    R_dc -> Vbias [style=dashed];
    Amp2 -> Rf [dir=back, label="feedback"];
    Rf -> Rg;
    Rg -> Vbias [style=dashed];
    Amp2 -> Clamp;
    Clamp -> R_load;
    R_load -> Output;
    
    // Power connections
    VDD -> Clamp [style=dotted, color=blue];
    VSS -> Clamp [style=dotted, color=blue];
    Vbias -> Buffer1 [style=dashed, color=green, label="bias"];
    
    // Ground connections
    Vpiezo -> VSS [style=dotted, color=gray];
    R_load -> VSS [style=dotted, color=gray];
}
"""
    
    with open('circuit_diagram.dot', 'w') as f:
        f.write(dot_content)
    
    print("Graphviz DOT file created: circuit_diagram.dot")
    print("\nTo generate images, run:")
    print("  dot -Tpng circuit_diagram.dot -o circuit_diagram.png")
    print("  dot -Tsvg circuit_diagram.dot -o circuit_diagram.svg")
    print("  dot -Tpdf circuit_diagram.dot -o circuit_diagram.pdf")

def generate_ascii_art():
    """Generate ASCII art representation of the circuit"""
    
    ascii_circuit = """
    Piezo Mic Preamp Circuit - ASCII Diagram
    =========================================
    
    VDD (3.3V)
        |
        +--[100k]--+--[100k]-- VSS (0V)
                   |
                   +-- Vbias (1.65V)
                   |
                   |         Unity Gain        AC Coupling      Gain Stage (G=10)
    Piezo          |          Buffer                                    
      ~            |            ___              ||                 ___
     (~) ----||----+----+------|>  |----+-------||----+-----------|>  |----+---- Output
      |      1nF        |      |___|    |       1µF   |           |___|    |     (0-3.3V)
      |                 |               |             |              |      |
      |              [10MΩ]             |          [1MΩ]            /|\\     |
      |                 |               |             |              |     [|]
     VSS               VSS              |          Vbias          [9k]     |  [100k]
                                        |                           |      |     |
                                        +---------------------------+     [|]   VSS
                                                                          [1k]
                                                                           |
                                                                        Vbias
    
    Key Components:
    - Piezo: ~50mV signal source with 1nF capacitance
    - Input Buffer: High-Z unity gain buffer
    - AC Coupling: 1µF cap with 1MΩ DC restoration
    - Gain Stage: Non-inverting amp with gain of 10
    - Output: Clamped to 0-3.3V range
    """
    
    print(ascii_circuit)
    
    with open('circuit_ascii.txt', 'w') as f:
        f.write(ascii_circuit)
    print("\nASCII diagram saved to: circuit_ascii.txt")

if __name__ == "__main__":
    print("Generating circuit visualizations...")
    print("=" * 50)
    
    # Generate ASCII art
    generate_ascii_art()
    
    print("\n" + "=" * 50)
    
    # Generate Graphviz file
    generate_graphviz()