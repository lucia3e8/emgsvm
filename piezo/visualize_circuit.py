#!/usr/bin/env python3
import matplotlib.pyplot as plt
import matplotlib.patches as patches
from matplotlib.patches import Rectangle, Circle, FancyBboxPatch
import numpy as np

def draw_circuit():
    fig, ax = plt.subplots(1, 1, figsize=(14, 10))
    
    # Component positions
    components = {
        'Vpiezo': (1, 5),
        'Cpiezo': (2, 5),
        'Rpiezo': (3, 5),
        'Buffer1': (5, 5),
        'C_ac': (7, 5),
        'R_dc': (8, 4),
        'Amp2': (10, 5),
        'R_gain': (10, 3),
        'Diodes': (12, 5),
        'R_load': (13, 4),
        'Vbias': (6, 2),
        'VDD': (0.5, 8),
        'VSS': (0.5, 1)
    }
    
    # Draw components
    # Power supplies
    ax.text(components['VDD'][0], components['VDD'][1], 'VDD\n3.3V', 
            ha='center', va='center', bbox=dict(boxstyle="round,pad=0.3", facecolor='lightblue'))
    ax.text(components['VSS'][0], components['VSS'][1], 'VSS\n0V', 
            ha='center', va='center', bbox=dict(boxstyle="round,pad=0.3", facecolor='lightgray'))
    
    # Piezo source
    circle = Circle(components['Vpiezo'], 0.3, fill=False)
    ax.add_patch(circle)
    ax.text(components['Vpiezo'][0], components['Vpiezo'][1], '~', ha='center', va='center', fontsize=16)
    ax.text(components['Vpiezo'][0], components['Vpiezo'][1]-0.6, 'Vpiezo\n50mV', ha='center', va='top', fontsize=9)
    
    # Capacitors
    ax.plot([components['Cpiezo'][0]-0.1, components['Cpiezo'][0]-0.1], 
            [components['Cpiezo'][1]-0.2, components['Cpiezo'][1]+0.2], 'k-', linewidth=2)
    ax.plot([components['Cpiezo'][0]+0.1, components['Cpiezo'][0]+0.1], 
            [components['Cpiezo'][1]-0.2, components['Cpiezo'][1]+0.2], 'k-', linewidth=2)
    ax.text(components['Cpiezo'][0], components['Cpiezo'][1]-0.5, 'Cpiezo\n1nF', ha='center', va='top', fontsize=9)
    
    # Resistors (zigzag pattern)
    def draw_resistor(ax, pos, label):
        x, y = pos
        ax.plot([x-0.3, x-0.2, x-0.15, x-0.05, x+0.05, x+0.15, x+0.2, x+0.3], 
                [y, y, y+0.1, y-0.1, y+0.1, y-0.1, y, y], 'k-', linewidth=1.5)
        ax.text(x, y-0.4, label, ha='center', va='top', fontsize=9)
    
    draw_resistor(ax, components['Rpiezo'], 'Rpiezo\n10MΩ')
    draw_resistor(ax, components['R_dc'], 'R_dc\n1MΩ')
    draw_resistor(ax, components['R_load'], 'R_load\n100kΩ')
    
    # Op-amps (triangles)
    def draw_opamp(ax, pos, label):
        x, y = pos
        triangle = patches.Polygon([(x-0.4, y-0.3), (x-0.4, y+0.3), (x+0.4, y)], 
                                 closed=True, fill=False, linewidth=2)
        ax.add_patch(triangle)
        ax.text(x, y-0.6, label, ha='center', va='top', fontsize=9)
        return triangle
    
    draw_opamp(ax, components['Buffer1'], 'Buffer\nG=1')
    draw_opamp(ax, components['Amp2'], 'Amp\nG=10')
    
    # AC coupling capacitor
    ax.plot([components['C_ac'][0]-0.1, components['C_ac'][0]-0.1], 
            [components['C_ac'][1]-0.2, components['C_ac'][1]+0.2], 'k-', linewidth=2)
    ax.plot([components['C_ac'][0]+0.1, components['C_ac'][0]+0.1], 
            [components['C_ac'][1]-0.2, components['C_ac'][1]+0.2], 'k-', linewidth=2)
    ax.text(components['C_ac'][0], components['C_ac'][1]-0.5, 'C_ac\n1µF', ha='center', va='top', fontsize=9)
    
    # Gain resistors
    ax.text(components['R_gain'][0]-0.7, components['R_gain'][1], 'Rf=9k', ha='center', va='center', fontsize=9)
    ax.text(components['R_gain'][0]+0.7, components['R_gain'][1], 'Rg=1k', ha='center', va='center', fontsize=9)
    
    # Diode clamps
    ax.text(components['Diodes'][0], components['Diodes'][1], 'Clamp\nDiodes', 
            ha='center', va='center', bbox=dict(boxstyle="round,pad=0.3", facecolor='lightyellow'))
    
    # Vbias divider
    ax.text(components['Vbias'][0], components['Vbias'][1], 'Vbias\n1.65V', 
            ha='center', va='center', bbox=dict(boxstyle="round,pad=0.3", facecolor='lightgreen'))
    
    # Draw connections
    # Input path
    ax.plot([components['Vpiezo'][0]+0.3, components['Cpiezo'][0]-0.1], 
            [components['Vpiezo'][1], components['Cpiezo'][1]], 'k-')
    ax.plot([components['Cpiezo'][0]+0.1, components['Rpiezo'][0]-0.3], 
            [components['Cpiezo'][1], components['Rpiezo'][1]], 'k-')
    ax.plot([components['Rpiezo'][0]+0.3, components['Buffer1'][0]-0.4], 
            [components['Rpiezo'][1], components['Buffer1'][1]], 'k-')
    
    # Buffer to AC coupling
    ax.plot([components['Buffer1'][0]+0.4, components['C_ac'][0]-0.1], 
            [components['Buffer1'][1], components['C_ac'][1]], 'k-')
    
    # AC coupling to amp
    ax.plot([components['C_ac'][0]+0.1, components['Amp2'][0]-0.4], 
            [components['C_ac'][1], components['Amp2'][1]], 'k-')
    
    # R_dc connection
    ax.plot([components['C_ac'][0]+0.2, components['C_ac'][0]+0.2, components['R_dc'][0]], 
            [components['C_ac'][1], components['R_dc'][1]+0.3, components['R_dc'][1]+0.3], 'k-')
    
    # Amp to output
    ax.plot([components['Amp2'][0]+0.4, components['Diodes'][0]-0.3], 
            [components['Amp2'][1], components['Diodes'][1]], 'k-')
    ax.plot([components['Diodes'][0]+0.3, components['R_load'][0]], 
            [components['Diodes'][1], components['R_load'][1]+0.3], 'k-')
    
    # Vbias connections (dashed)
    ax.plot([components['Vbias'][0], components['Vbias'][0], components['R_dc'][0]], 
            [components['Vbias'][1]+0.3, components['R_dc'][1]-0.3, components['R_dc'][1]-0.3], 
            'k--', alpha=0.5)
    ax.plot([components['Vbias'][0], components['Vbias'][0], components['Buffer1'][0]], 
            [components['Vbias'][1]+0.3, components['Buffer1'][1]-0.3, components['Buffer1'][1]-0.3], 
            'k--', alpha=0.5)
    
    # Ground connections
    ax.plot([components['Vpiezo'][0], components['Vpiezo'][0]], 
            [components['Vpiezo'][1]-0.3, components['VSS'][1]+0.3], 'k-')
    ax.plot([components['R_load'][0], components['R_load'][0]], 
            [components['R_load'][1]-0.3, components['VSS'][1]+0.3], 'k-')
    
    # Labels
    ax.text(7, 7, 'Piezo Mic Preamp Circuit', fontsize=16, fontweight='bold', ha='center')
    ax.text(1, 6, 'Input\nStage', fontsize=11, ha='center', style='italic')
    ax.text(7, 6, 'AC Coupling', fontsize=11, ha='center', style='italic')
    ax.text(10, 6, 'Gain Stage', fontsize=11, ha='center', style='italic')
    ax.text(13, 6, 'Output', fontsize=11, ha='center', style='italic')
    
    # Annotations
    ax.annotate('High-Z Input', xy=(4, 5), xytext=(4, 7),
                arrowprops=dict(arrowstyle='->', color='blue', alpha=0.7))
    ax.annotate('0-3.3V Output', xy=(13, 4.5), xytext=(13, 2.5),
                arrowprops=dict(arrowstyle='->', color='red', alpha=0.7))
    
    # Set limits and remove axes
    ax.set_xlim(-0.5, 14.5)
    ax.set_ylim(0, 8.5)
    ax.set_aspect('equal')
    ax.axis('off')
    
    # Grid for alignment (optional)
    # ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig('circuit_diagram.png', dpi=300, bbox_inches='tight')
    plt.savefig('circuit_diagram.pdf', bbox_inches='tight')
    print("Circuit diagram saved as circuit_diagram.png and circuit_diagram.pdf")

if __name__ == "__main__":
    draw_circuit()