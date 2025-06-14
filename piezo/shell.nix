{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    ngspice
    python3
    python311Packages.matplotlib
    python311Packages.numpy
    gnuplot
    graphviz
  ];

  shellHook = ''
    echo "SPICE environment loaded!"
    echo "Available tools:"
    echo "  - ngspice: SPICE circuit simulator"
    echo "  - python3: For data processing and plotting"
    echo "  - gnuplot: For quick plots"
    echo "  - graphviz: For circuit diagram generation"
    echo ""
    echo "To run a simulation: ngspice circuit.cir"
    echo "To run interactively: ngspice"
  '';
}